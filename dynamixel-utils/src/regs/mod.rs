mod db;

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

use crate::protocol::ProtocolVersion;
use db::REGS;

#[derive(Debug, Clone, Copy)]
pub enum Access {
    R,
    W,
    RW,
}

impl Display for Access {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Access::R => "R".fmt(f),
            Access::W => "W".fmt(f),
            Access::RW => "RW".fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RegSize {
    Byte = 1,
    Half = 2,
    Word = 4,
    Variable = 0,
}

#[derive(Debug, Clone, Copy)]
pub struct Reg {
    pub model: &'static str,
    pub proto: ProtocolVersion,
    pub name: &'static str,
    pub address: u16,
    pub size: RegSize,
    pub access: Access,
}

impl Reg {
    pub const fn new(
        model: &'static str,
        proto: ProtocolVersion,
        name: &'static str,
        address: u16,
        size: RegSize,
        access: Access,
    ) -> Self {
        Reg {
            model,
            proto,
            name,
            address,
            size,
            access,
        }
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:4} {:1} {:<2} {}",
            self.address, self.size as u8, self.access, self.name
        )
    }
}

#[derive(Debug)]
pub struct RegSpec {
    pub model: String,
    pub name: String,
}

#[derive(Error, Debug)]
pub enum RegSpecError {
    #[error("invalid register specification")]
    BadRegSpec,
}

impl FromStr for RegSpec {
    type Err = RegSpecError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"^([-_[:alnum:]]+)/([-_[:alnum:]]+)$").unwrap();
        }
        if let Some(cap) = RE.captures(s) {
            Ok(RegSpec {
                model: cap.get(1).unwrap().as_str().to_string(),
                name: cap.get(2).unwrap().as_str().to_string(),
            })
        } else {
            Err(RegSpecError::BadRegSpec)
        }
    }
}

pub fn list_models(proto: ProtocolVersion) -> Vec<&'static str> {
    REGS.iter()
        .filter(|reg| reg.proto == proto)
        .map(|reg| reg.model)
        .unique()
        .sorted()
        .collect()
}

pub fn list_registers(proto: ProtocolVersion, model: &str) -> Vec<Reg> {
    REGS.iter()
        .cloned()
        .filter(|reg| reg.model == model && reg.proto == proto)
        .collect()
}

pub fn find_register(proto: ProtocolVersion, regspec: RegSpec) -> Option<Reg> {
    REGS.iter()
        .cloned()
        .filter(|reg| reg.proto == proto && reg.model == regspec.model && reg.name == regspec.name)
        .take(1)
        .next()
}
