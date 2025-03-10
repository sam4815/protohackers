use std::{
    collections::HashMap,
    io::{self, BufReader, Error, ErrorKind},
    net::TcpStream,
    sync::mpsc,
};

use crate::{
    models::{
        CreatePolicy, DeletePolicy, DialAuthority, Hello, Message, SiteVisit, VisitPopulation,
    },
    read::consume_messages,
    write::write_message,
};
use PolicyAction::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PolicyAction {
    Cull = 0x90,
    Conserve = 0xa0,
    DoNothing = 0x00,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Policy {
    pub id: u32,
    pub action: PolicyAction,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Target {
    pub min: u32,
    pub max: u32,
}

pub struct Site {
    pub receiver: mpsc::Receiver<SiteVisit>,
    pub stream: TcpStream,
    pub targets: HashMap<String, Target>,
    pub policies: HashMap<String, Policy>,
}

impl Site {
    pub fn new(site_id: u32, receiver: mpsc::Receiver<SiteVisit>) -> io::Result<Site> {
        let stream = TcpStream::connect("pestcontrol.protohackers.com:20547")?;
        let mut reader = consume_messages(BufReader::new(&stream));
        let mut writer = &stream;

        write_message(
            &mut writer,
            Message::Hello(Hello {
                protocol: "pestcontrol".to_string(),
                version: 1,
            }),
        )?;

        match reader.next() {
            Some(Ok(Message::Hello(message)))
                if message.protocol == "pestcontrol" && message.version == 1 => {}
            Some(Ok(Message::PestControlError(e))) => {
                return Err(Error::new(ErrorKind::InvalidInput, e.message))
            }
            Some(Err(e)) => return Err(e),
            _ => return Err(Error::from(ErrorKind::InvalidData)),
        }

        write_message(
            &mut writer,
            Message::DialAuthority(DialAuthority { site: site_id }),
        )?;

        match reader.next() {
            Some(Ok(Message::TargetPopulations(message))) if message.site == site_id => {
                let mut site = Site {
                    receiver,
                    stream,
                    targets: HashMap::new(),
                    policies: HashMap::new(),
                };

                for population in message.populations {
                    site.targets.insert(
                        population.species,
                        Target {
                            min: population.min,
                            max: population.max,
                        },
                    );
                }

                Ok(site)
            }
            Some(Ok(Message::PestControlError(e))) => {
                Err(Error::new(ErrorKind::InvalidInput, e.message))
            }
            Some(Err(e)) => Err(e),
            _ => Err(Error::from(ErrorKind::InvalidData)),
        }
    }

    pub fn create_policy(&mut self, species: String, action: PolicyAction) -> io::Result<()> {
        write_message(
            &mut self.stream,
            Message::CreatePolicy(CreatePolicy {
                species: species.clone(),
                action: action as u8,
            }),
        )?;

        match consume_messages(BufReader::new(&self.stream)).next() {
            Some(Ok(Message::PolicyResult(message))) => {
                self.policies.insert(
                    species.clone(),
                    Policy {
                        id: message.policy,
                        action,
                    },
                );

                Ok(())
            }
            Some(Ok(Message::PestControlError(e))) => {
                Err(Error::new(ErrorKind::InvalidInput, e.message))
            }
            Some(Err(e)) => Err(e),
            _ => Err(Error::from(ErrorKind::InvalidData)),
        }
    }

    pub fn delete_policy(&mut self, species: String) -> io::Result<()> {
        write_message(
            &mut self.stream,
            Message::DeletePolicy(DeletePolicy {
                policy: self.policies.get(&species).unwrap().id,
            }),
        )?;

        match consume_messages(BufReader::new(&self.stream)).next() {
            Some(Ok(Message::Okay(_))) => {
                self.policies.remove(&species);
                Ok(())
            }
            Some(Ok(Message::PestControlError(e))) => {
                Err(Error::new(ErrorKind::InvalidInput, e.message))
            }
            Some(Err(e)) => Err(e),
            _ => Err(Error::from(ErrorKind::InvalidData)),
        }
    }

    pub fn poll(&mut self) -> io::Result<()> {
        while let Ok(visit) = self.receiver.recv() {
            let targets = self.targets.clone();
            let policies = self.policies.clone();

            let sightings: HashMap<String, VisitPopulation> = visit
                .populations
                .into_iter()
                .map(|n| (n.species.clone(), n))
                .collect();

            for (species, target) in targets.iter() {
                let recommended_policy = match sightings.get(species) {
                    Some(population) if population.count > target.max => Cull,
                    Some(population) if population.count < target.min => Conserve,
                    Some(_) => DoNothing,
                    None => Conserve,
                };
                let existing_policy = policies.get(species);

                match (recommended_policy, existing_policy) {
                    (Conserve, None) | (Cull, None) => {
                        self.create_policy(species.clone(), recommended_policy)?;
                    }
                    (DoNothing, None) => {}
                    (_, Some(policy)) => {
                        if policy.action != recommended_policy {
                            self.delete_policy(species.clone())?;
                            self.create_policy(species.clone(), recommended_policy)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
