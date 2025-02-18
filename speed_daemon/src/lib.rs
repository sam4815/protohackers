mod serialization;

use std::collections::HashMap;
use serialization::models::{LengthPrefixedString, Ticket};

#[derive(Clone, Debug, PartialEq)]
pub struct Sighting {
    pub plate: String,
    pub timestamp: u32,
    pub road: u16,
    pub mile: u16,
    pub limit: u16,
}

#[derive(Clone, Debug)]
pub struct Road {
    pub id: u16,
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
    pub ticket_days: Vec<u32>
}

#[derive(Clone, Debug)]
pub enum SpeedMessage {
    TicketRequired(Ticket),
    TicketSent(Ticket),
    Sighting(Sighting),
}

impl Car {
    pub fn new(plate: String) -> Self {
        let roads = HashMap::<u16, Road>::new();
        let sightings = HashMap::<u16, Vec<Point>>::new();
        let ticket_days = Vec::<u32>::new();

        Car { plate, roads, sightings, ticket_days }
    }

    pub fn add_sighting(&mut self, sighting: Sighting) {
        match self.sightings.get_mut(&sighting.road) {
            Some(sightings) => {
                sightings.insert(0, Point{mile: sighting.mile, timestamp: sighting.timestamp});
                sightings.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            }
            None => {
                self.roads.insert(
                    sighting.road,
                    Road{ id: sighting.road, limit: sighting.limit },
                );
                self.sightings.insert(
                    sighting.road,
                    vec![Point{mile: sighting.mile, timestamp: sighting.timestamp}]
                );
            }
        }
    }

    pub fn get_infractions(&mut self) -> Vec<Ticket> {
        let mut tickets = Vec::new();

        for (road_id, sightings) in self.sightings.iter() {
            for window in sightings.windows(2) {
                if let [first, second] = window {
                    let distance: f32 = (second.mile - first.mile).into();
                    let seconds: f64 = (second.timestamp - first.timestamp).into();
                    let mph = (3600.0/seconds as f32) * distance;

                    if let Some(road) = self.roads.get(road_id) {
                        if mph > road.limit as f32 + 0.5 {
                            tickets.insert(0, Ticket{
                                road: *road_id,
                                plate: LengthPrefixedString(self.plate.clone()),
                                mile1: first.mile,
                                timestamp1: first.timestamp,
                                mile2: second.mile,
                                timestamp2: second.timestamp,
                                speed: (mph * 100.0) as u16,
                            });
                        }
                    }
                }
            }
        }

        tickets
    }

    pub fn get_outstanding_tickets(&mut self) -> Vec<Ticket> {
        let mut tickets = Vec::new();

        for ticket in self.get_infractions() {
            let day1 = (ticket.timestamp1 as f32 / 86400.0).floor() as u32;
            let day2 = (ticket.timestamp2 as f32 / 86400.0).floor() as u32;

            if !self.ticket_days.contains(&day1) {
                tickets.insert(0, ticket.clone());
            }
            if !self.ticket_days.contains(&day2) {
                tickets.insert(0, ticket.clone());
            }
        }

        tickets
    }

    pub fn mark_ticket_dispatched(&mut self, ticket: Ticket) {
        let day1 = (ticket.timestamp1 as f32 / 86400.0).floor() as u32;
        let day2 = (ticket.timestamp2 as f32 / 86400.0).floor() as u32;

        self.ticket_days.insert(0, day1);
        self.ticket_days.insert(0, day2);
    }
}
