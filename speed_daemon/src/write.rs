use std::io::Write;

use super::models::Ticket;

fn write_char(writer: &mut impl Write, character: u8) -> std::io::Result<()> {
    writer.write_all(&[character])
}

fn write_string(writer: &mut impl Write, string: String) -> std::io::Result<()> {
    let mut buffer = string.clone().into_bytes();
    let size = string.clone().len().to_le_bytes();
    buffer.insert(0, size[0]);

    writer.write_all(&buffer)
}

fn write_u16(writer: &mut impl Write, number: u16) -> std::io::Result<()> {
    writer.write_all(&number.to_be_bytes())
}

fn write_u32(writer: &mut impl Write, number: u32) -> std::io::Result<()> {
    writer.write_all(&number.to_be_bytes())
}

pub fn write_ticket(writer: &mut impl Write, ticket: Ticket) -> std::io::Result<()> {
    write_char(writer, 0x21)?;
    write_string(writer, ticket.plate)?;
    write_u16(writer, ticket.road)?;
    write_u16(writer, ticket.mile1)?;
    write_u32(writer, ticket.timestamp1)?;
    write_u16(writer, ticket.mile2)?;
    write_u32(writer, ticket.timestamp2)?;
    write_u16(writer, ticket.speed)
}

pub fn write_heartbeat(writer: &mut impl Write) -> std::io::Result<()> {
    write_char(writer, 0x41)
}

pub fn write_error(writer: &mut impl Write, error: String) -> std::io::Result<()> {
    write_char(writer, 0x20)?;
    write_string(writer, error)
}
