use crate::models::{Sighting, Ticket};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Road {
    pub limit: u16,
}

pub struct Point {
    pub mile: u16,
    pub timestamp: u32,
}

pub struct Car {
    pub plate: String,
    pub roads: HashMap<u16, Road>,
    pub sightings: HashMap<u16, Vec<Point>>,
    pub ticket_days: HashSet<u32>,
}

impl Car {
    pub fn new(plate: String) -> Self {
        let roads = HashMap::<u16, Road>::new();
        let sightings = HashMap::<u16, Vec<Point>>::new();
        let ticket_days = HashSet::<u32>::new();

        Car {
            plate,
            roads,
            sightings,
            ticket_days,
        }
    }

    pub fn add_sighting(&mut self, sighting: Sighting) {
        match self.sightings.get_mut(&sighting.road) {
            Some(sightings) => {
                sightings.insert(
                    0,
                    Point {
                        mile: sighting.mile,
                        timestamp: sighting.timestamp,
                    },
                );
                sightings.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            }
            None => {
                self.roads.insert(
                    sighting.road,
                    Road {
                        limit: sighting.limit,
                    },
                );
                self.sightings.insert(
                    sighting.road,
                    vec![Point {
                        mile: sighting.mile,
                        timestamp: sighting.timestamp,
                    }],
                );
            }
        }
    }

    pub fn get_infractions(&mut self) -> Vec<Ticket> {
        let mut tickets = Vec::new();

        for (road_id, sightings) in self.sightings.iter() {
            for window in sightings.windows(2) {
                if let [first, second] = window {
                    let distance: f32 = second.mile.abs_diff(first.mile).into();
                    let seconds: f64 = (second.timestamp - first.timestamp).into();
                    let mph = (3600.0 / seconds as f32) * distance;

                    if let Some(road) = self.roads.get(road_id) {
                        if mph > road.limit as f32 + 0.5 {
                            tickets.insert(
                                0,
                                Ticket {
                                    road: *road_id,
                                    plate: self.plate.clone(),
                                    mile1: first.mile,
                                    timestamp1: first.timestamp,
                                    mile2: second.mile,
                                    timestamp2: second.timestamp,
                                    speed: (mph * 100.0) as u16,
                                },
                            );
                        }
                    }
                }
            }
        }

        tickets
    }

    pub fn get_outstanding_tickets(&mut self) -> Vec<Ticket> {
        let mut tickets = Vec::new();

        let mut ticket_days_sent = HashSet::<u32>::new();
        for day in self.ticket_days.iter().copied() {
            ticket_days_sent.insert(day);
        }

        for ticket in self.get_infractions() {
            let day1 = (ticket.timestamp1 as f32 / 86400.0).floor() as u32;
            let day2 = (ticket.timestamp2 as f32 / 86400.0).floor() as u32;

            if !ticket_days_sent.contains(&day1) && !ticket_days_sent.contains(&day2) {
                tickets.insert(0, ticket.clone());
                ticket_days_sent.insert(day1);
                ticket_days_sent.insert(day2);
            }
        }

        tickets
    }

    pub fn mark_ticket_dispatched(&mut self, ticket: Ticket) {
        let day1 = (ticket.timestamp1 as f32 / 86400.0).floor() as u32;
        let day2 = (ticket.timestamp2 as f32 / 86400.0).floor() as u32;

        self.ticket_days.insert(day1);
        self.ticket_days.insert(day2);
    }
}
