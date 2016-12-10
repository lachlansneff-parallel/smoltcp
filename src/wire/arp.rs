use core::fmt;
use byteorder::{ByteOrder, NetworkEndian};

pub use super::EthernetProtocolType as ProtocolType;

enum_with_unknown! {
    /// ARP network protocol type.
    pub enum HardwareType(u16) {
        Ethernet = 1
    }
}

enum_with_unknown! {
    /// ARP operation type.
    pub enum Operation(u16) {
        Request = 1,
        Reply = 2
    }
}

/// A read/write wrapper around an Address Resolution Protocol packet.
#[derive(Debug)]
pub struct Packet<T: AsRef<[u8]>>(T);

mod field {
    #![allow(non_snake_case)]

    use ::wire::field::*;

    pub const HTYPE: Field = 0..2;
    pub const PTYPE: Field = 2..4;
    pub const HLEN:  usize = 4;
    pub const PLEN:  usize = 5;
    pub const OPER:  Field = 6..8;

    #[inline(always)]
    pub fn SHA(hardware_length: u8, _protocol_length: u8) -> Field {
        let start = OPER.end;
        start..(start + hardware_length as usize)
    }

    #[inline(always)]
    pub fn SPA(hardware_length: u8, protocol_length: u8) -> Field {
        let start = SHA(hardware_length, protocol_length).end;
        start..(start + protocol_length as usize)
    }

    #[inline(always)]
    pub fn THA(hardware_length: u8, protocol_length: u8) -> Field {
        let start = SPA(hardware_length, protocol_length).end;
        start..(start + hardware_length as usize)
    }

    #[inline(always)]
    pub fn TPA(hardware_length: u8, protocol_length: u8) -> Field {
        let start = THA(hardware_length, protocol_length).end;
        start..(start + protocol_length as usize)
    }
}

impl<T: AsRef<[u8]>> Packet<T> {
    /// Wrap a buffer with an ARP packet. Returns an error if the buffer
    /// is too small to contain one.
    pub fn new(storage: T) -> Result<Packet<T>, ()> {
        let len = storage.as_ref().len();
        if len < field::OPER.end {
            Err(())
        } else {
            let packet = Packet(storage);
            if len < field::TPA(packet.hardware_length(), packet.protocol_length()).end {
                Err(())
            } else {
                Ok(packet)
            }
        }
    }

    /// Consumes the packet, returning the underlying buffer.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Return the hardware type field.
    pub fn hardware_type(&self) -> HardwareType {
        let bytes = self.0.as_ref();
        let raw = NetworkEndian::read_u16(&bytes[field::HTYPE]);
        HardwareType::from(raw)
    }

    /// Return the protocol type field.
    pub fn protocol_type(&self) -> ProtocolType {
        let bytes = self.0.as_ref();
        let raw = NetworkEndian::read_u16(&bytes[field::PTYPE]);
        ProtocolType::from(raw)
    }

    /// Return the hardware length field.
    pub fn hardware_length(&self) -> u8 {
        let bytes = self.0.as_ref();
        bytes[field::HLEN]
    }

    /// Return the protocol length field.
    pub fn protocol_length(&self) -> u8 {
        let bytes = self.0.as_ref();
        bytes[field::PLEN]
    }

    /// Return the operation field.
    pub fn operation(&self) -> Operation {
        let bytes = self.0.as_ref();
        let raw = NetworkEndian::read_u16(&bytes[field::OPER]);
        Operation::from(raw)
    }

    /// Return the source hardware address field.
    pub fn source_hardware_addr(&self) -> &[u8] {
        let bytes = self.0.as_ref();
        &bytes[field::SHA(self.hardware_length(), self.protocol_length())]
    }

    /// Return the source protocol address field.
    pub fn source_protocol_addr(&self) -> &[u8] {
        let bytes = self.0.as_ref();
        &bytes[field::SPA(self.hardware_length(), self.protocol_length())]
    }

    /// Return the target hardware address field.
    pub fn target_hardware_addr(&self) -> &[u8] {
        let bytes = self.0.as_ref();
        &bytes[field::THA(self.hardware_length(), self.protocol_length())]
    }

    /// Return the target protocol address field.
    pub fn target_protocol_addr(&self) -> &[u8] {
        let bytes = self.0.as_ref();
        &bytes[field::TPA(self.hardware_length(), self.protocol_length())]
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> Packet<T> {
    /// Set the hardware type field.
    pub fn set_hardware_type(&mut self, value: HardwareType) {
        let bytes = self.0.as_mut();
        NetworkEndian::write_u16(&mut bytes[field::HTYPE], value.into())
    }

    /// Set the protocol type field.
    pub fn set_protocol_type(&mut self, value: ProtocolType) {
        let bytes = self.0.as_mut();
        NetworkEndian::write_u16(&mut bytes[field::PTYPE], value.into())
    }

    /// Set the hardware length field.
    pub fn set_hardware_length(&mut self, value: u8) {
        let bytes = self.0.as_mut();
        bytes[field::HLEN] = value
    }

    /// Set the protocol length field.
    pub fn set_protocol_length(&mut self, value: u8) {
        let bytes = self.0.as_mut();
        bytes[field::PLEN] = value
    }

    /// Set the operation field.
    pub fn set_operation(&mut self, value: Operation) {
        let bytes = self.0.as_mut();
        NetworkEndian::write_u16(&mut bytes[field::OPER], value.into())
    }

    /// Set the source hardware address field.
    ///
    /// # Panics
    /// The function panics if `value` is not `self.hardware_length()` long.
    pub fn set_source_hardware_addr(&mut self, value: &[u8]) {
        let (hardware_length, protocol_length) = (self.hardware_length(), self.protocol_length());
        let bytes = self.0.as_mut();
        bytes[field::SHA(hardware_length, protocol_length)].copy_from_slice(value)
    }

    /// Set the source protocol address field.
    ///
    /// # Panics
    /// The function panics if `value` is not `self.protocol_length()` long.
    pub fn set_source_protocol_addr(&mut self, value: &[u8]) {
        let (hardware_length, protocol_length) = (self.hardware_length(), self.protocol_length());
        let bytes = self.0.as_mut();
        bytes[field::SPA(hardware_length, protocol_length)].copy_from_slice(value)
    }

    /// Set the target hardware address field.
    ///
    /// # Panics
    /// The function panics if `value` is not `self.hardware_length()` long.
    pub fn set_target_hardware_addr(&mut self, value: &[u8]) {
        let (hardware_length, protocol_length) = (self.hardware_length(), self.protocol_length());
        let bytes = self.0.as_mut();
        bytes[field::THA(hardware_length, protocol_length)].copy_from_slice(value)
    }

    /// Set the target protocol address field.
    ///
    /// # Panics
    /// The function panics if `value` is not `self.protocol_length()` long.
    pub fn set_target_protocol_addr(&mut self, value: &[u8]) {
        let (hardware_length, protocol_length) = (self.hardware_length(), self.protocol_length());
        let bytes = self.0.as_mut();
        bytes[field::TPA(hardware_length, protocol_length)].copy_from_slice(value)
    }
}

impl<T: AsRef<[u8]>> fmt::Display for Packet<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match Repr::parse(self) {
            Ok(repr) => write!(f, "{}", repr),
            _ => {
                try!(write!(f, "ARP htype={:?} ptype={:?} hlen={:?} plen={:?} op={:?}",
                            self.hardware_type(), self.protocol_type(),
                            self.hardware_length(), self.protocol_length(),
                            self.operation()));
                try!(write!(f, " sha={:?} spa={:?} tha={:?} tpa={:?}",
                            self.source_hardware_addr(), self.source_protocol_addr(),
                            self.target_hardware_addr(), self.target_protocol_addr()));
                Ok(())
            }
        }
    }
}

use super::{EthernetAddress, Ipv4Address};

/// A high-level representation of an Address Resolution Protocol packet.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Repr {
    /// An Ethernet and IPv4 Address Resolution Protocol packet.
    EthernetIpv4 {
        operation: Operation,
        source_hardware_addr: EthernetAddress,
        source_protocol_addr: Ipv4Address,
        target_hardware_addr: EthernetAddress,
        target_protocol_addr: Ipv4Address
    },
    #[doc(hidden)]
    __Nonexhaustive
}

impl Repr {
    /// Parse an Address Resolution Packet and return a high-level representation,
    /// or return `Err(())` if the packet is not recognized.
    pub fn parse<T: AsRef<[u8]>>(packet: &Packet<T>) -> Result<Repr, ()> {
        match (packet.hardware_type(), packet.protocol_type(),
               packet.hardware_length(), packet.protocol_length()) {
            (HardwareType::Ethernet, ProtocolType::Ipv4, 6, 4) => {
                Ok(Repr::EthernetIpv4 {
                    operation: packet.operation(),
                    source_hardware_addr:
                        EthernetAddress::from_bytes(packet.source_hardware_addr()),
                    source_protocol_addr:
                        Ipv4Address::from_bytes(packet.source_protocol_addr()),
                    target_hardware_addr:
                        EthernetAddress::from_bytes(packet.target_hardware_addr()),
                    target_protocol_addr:
                        Ipv4Address::from_bytes(packet.target_protocol_addr())
                })
            },
            _ => Err(())
        }
    }

    /// Emit a high-level representation into an Address Resolution Packet.
    pub fn emit<T: AsRef<[u8]> + AsMut<[u8]>>(&self, packet: &mut Packet<T>) {
        match self {
            &Repr::EthernetIpv4 {
                operation,
                source_hardware_addr, source_protocol_addr,
                target_hardware_addr, target_protocol_addr
            } => {
                packet.set_hardware_type(HardwareType::Ethernet);
                packet.set_protocol_type(ProtocolType::Ipv4);
                packet.set_hardware_length(6);
                packet.set_protocol_length(4);
                packet.set_operation(operation);
                packet.set_source_hardware_addr(source_hardware_addr.as_bytes());
                packet.set_source_protocol_addr(source_protocol_addr.as_bytes());
                packet.set_target_hardware_addr(target_hardware_addr.as_bytes());
                packet.set_target_protocol_addr(target_protocol_addr.as_bytes());
            },
            &Repr::__Nonexhaustive => unreachable!()
        }
    }
}

impl fmt::Display for Repr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Repr::EthernetIpv4 {
                operation,
                source_hardware_addr, source_protocol_addr,
                target_hardware_addr, target_protocol_addr
            } => {
                write!(f, "ARP type=Ethernet+IPv4 src={}/{} dst={}/{} op={:?}",
                       source_hardware_addr, source_protocol_addr,
                       target_hardware_addr, target_protocol_addr,
                       operation)
            },
            &Repr::__Nonexhaustive => unreachable!()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    static PACKET_BYTES: [u8; 28] =
        [0x00, 0x01,
         0x08, 0x00,
         0x06,
         0x04,
         0x00, 0x01,
         0x11, 0x12, 0x13, 0x14, 0x15, 0x16,
         0x21, 0x22, 0x23, 0x24,
         0x31, 0x32, 0x33, 0x34, 0x35, 0x36,
         0x41, 0x42, 0x43, 0x44];

    #[test]
    fn test_deconstruct() {
        let packet = Packet::new(&PACKET_BYTES[..]).unwrap();
        assert_eq!(packet.hardware_type(), HardwareType::Ethernet);
        assert_eq!(packet.protocol_type(), ProtocolType::Ipv4);
        assert_eq!(packet.hardware_length(), 6);
        assert_eq!(packet.protocol_length(), 4);
        assert_eq!(packet.operation(), Operation::Request);
        assert_eq!(packet.source_hardware_addr(), &[0x11, 0x12, 0x13, 0x14, 0x15, 0x16]);
        assert_eq!(packet.source_protocol_addr(), &[0x21, 0x22, 0x23, 0x24]);
        assert_eq!(packet.target_hardware_addr(), &[0x31, 0x32, 0x33, 0x34, 0x35, 0x36]);
        assert_eq!(packet.target_protocol_addr(), &[0x41, 0x42, 0x43, 0x44]);
    }

    #[test]
    fn test_construct() {
        let mut bytes = vec![0; 28];
        let mut packet = Packet::new(&mut bytes).unwrap();
        packet.set_hardware_type(HardwareType::Ethernet);
        packet.set_protocol_type(ProtocolType::Ipv4);
        packet.set_hardware_length(6);
        packet.set_protocol_length(4);
        packet.set_operation(Operation::Request);
        packet.set_source_hardware_addr(&[0x11, 0x12, 0x13, 0x14, 0x15, 0x16]);
        packet.set_source_protocol_addr(&[0x21, 0x22, 0x23, 0x24]);
        packet.set_target_hardware_addr(&[0x31, 0x32, 0x33, 0x34, 0x35, 0x36]);
        packet.set_target_protocol_addr(&[0x41, 0x42, 0x43, 0x44]);
        assert_eq!(&packet.into_inner()[..], &PACKET_BYTES[..]);
    }

    fn packet_repr() -> Repr {
        Repr::EthernetIpv4 {
            operation: Operation::Request,
            source_hardware_addr:
                EthernetAddress::from_bytes(&[0x11, 0x12, 0x13, 0x14, 0x15, 0x16]),
            source_protocol_addr:
                Ipv4Address::from_bytes(&[0x21, 0x22, 0x23, 0x24]),
            target_hardware_addr:
                EthernetAddress::from_bytes(&[0x31, 0x32, 0x33, 0x34, 0x35, 0x36]),
            target_protocol_addr:
                Ipv4Address::from_bytes(&[0x41, 0x42, 0x43, 0x44])
        }
    }

    #[test]
    fn test_parse() {
        let packet = Packet::new(&PACKET_BYTES[..]).unwrap();
        let repr = Repr::parse(&packet).unwrap();
        assert_eq!(repr, packet_repr());
    }

    #[test]
    fn test_emit() {
        let mut bytes = vec![0; 28];
        let mut packet = Packet::new(&mut bytes).unwrap();
        packet_repr().emit(&mut packet);
        assert_eq!(&packet.into_inner()[..], &PACKET_BYTES[..]);
    }
}