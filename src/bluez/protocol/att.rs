use nom::{le_u8, le_u16, IResult};

use ::api::peripheral::{Characteristic, CharacteristicUUID, CharPropFlags};

use bluez::constants::*;
use bluez::protocol::*;

#[cfg(test)]
mod tests {
    use nom::IResult;
    use super::*;

    #[test]
    fn test_characteristics() {
        let buf = [9, 7, 2, 0, 2, 3, 0, 0, 42, 4, 0, 2, 5, 0, 1, 42, 6, 0, 10, 7, 0, 2, 42];
        assert_eq!(characteristics(&buf), IResult::Done(
            &[][..],
            vec![
                Characteristic {
                    start_handle: 2,
                    value_handle: 3,
                    end_handle: 0xFFFF,
                    uuid: CharacteristicUUID::B16(0x2A00),
                    properties: CharPropFlags::READ
                },
                Characteristic {
                    start_handle: 4,
                    value_handle: 5,
                    end_handle: 0xFFFF,
                    uuid: CharacteristicUUID::B16(0x2A01),
                    properties: CharPropFlags::READ
                },
                Characteristic {
                    start_handle: 6,
                    value_handle: 7,
                    end_handle: 0xFFFF,
                    uuid: CharacteristicUUID::B16(0x2A02),
                    properties: CharPropFlags::READ | CharPropFlags::WRITE
                },
            ]
        ))
    }
}

#[derive(Debug, PartialEq)]
pub struct NotifyResponse {
    pub typ: u8,
    pub handle: u16,
    pub value: u16,
}

named!(pub notify_response<&[u8], NotifyResponse>,
   do_parse!(
      op: tag!(&[ATT_OP_READ_BY_TYPE_RESP]) >>
      typ: le_u8 >>
      handle: le_u16 >>
      value: le_u16 >>
      (
        NotifyResponse { typ, handle, value }
      )
   ));


fn characteristic(i: &[u8], b16_uuid: bool) -> IResult<&[u8], Characteristic> {
    let (i, start_handle) = try_parse!(i, le_u16);
    let (i, properties) = try_parse!(i, le_u8);
    let (i, value_handle) = try_parse!(i, le_u16);
    let (i, uuid) = if b16_uuid {
        try_parse!(i, map!(le_u16, |b| CharacteristicUUID::B16(b)))
    } else {
        try_parse!(i, map!(parse_uuid_128, |b| CharacteristicUUID::B128(b)))
    };

    IResult::Done(i, Characteristic {
        start_handle,
        value_handle,
        end_handle: 0xFFFF,
        uuid,
        properties: CharPropFlags::from_bits_truncate(properties),
    })
}

pub fn characteristics(i: &[u8]) -> IResult<&[u8], Vec<Characteristic>> {
    let (i, opcode) = try_parse!(i, le_u8);

    let (i, result) = match opcode {
        ATT_OP_READ_BY_TYPE_RESP => {
            let (i, rec_len) = try_parse!(i, le_u8);
            let num = i.len() / rec_len as usize;
            let b16_uuid = rec_len == 7;
            try_parse!(i, count!(apply!(characteristic, b16_uuid), num))
        }
        x => {
            warn!("unhandled characteristics op type {}", x);
            (&[][..], vec![])
        }
    };

    IResult::Done(i, result)
}
