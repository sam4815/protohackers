use std::io::BufReader;

#[derive(Clone, PartialEq, Debug)]
pub struct Hello {
    pub protocol: String,
    pub version: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PestControlError {
    pub message: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Okay {}

#[derive(Clone, PartialEq, Debug)]
pub struct DialAuthority {
    pub site: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TargetPopulation {
    pub species: String,
    pub min: u32,
    pub max: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TargetPopulations {
    pub site: u32,
    pub populations: Vec<TargetPopulation>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct CreatePolicy {
    pub species: String,
    pub action: u8,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DeletePolicy {
    pub policy: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct PolicyResult {
    pub policy: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct VisitPopulation {
    pub species: String,
    pub count: u32,
}

#[derive(Clone, PartialEq, Debug)]
pub struct SiteVisit {
    pub site: u32,
    pub populations: Vec<VisitPopulation>,
}

#[derive(Clone, Debug)]
pub enum Message {
    Hello(Hello),
    PestControlError(PestControlError),
    Okay(Okay),
    DialAuthority(DialAuthority),
    TargetPopulations(TargetPopulations),
    CreatePolicy(CreatePolicy),
    DeletePolicy(DeletePolicy),
    PolicyResult(PolicyResult),
    SiteVisit(SiteVisit),
}

pub struct MessageIterator<R> {
    pub reader: BufReader<R>,
}
