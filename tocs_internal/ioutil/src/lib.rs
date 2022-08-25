pub mod pkg;
pub mod tbl;

#[derive(Clone, Debug)]
pub struct ParseContext<'input> {
    pub strict: bool,
    input: &'input [u8],
    position: usize,
}

impl<'a> ParseContext<'a> {
    pub fn start_of(input: &'a [u8]) -> Self {
        Self { strict: true, input, position: 0 }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn less_safe(&self) -> Self {
        let mut ctx = self.clone();
        ctx.strict = false;
        ctx
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrimitiveParseError {
    UnexpectedEOF { position: usize }, // expected more data, but reached the end
    MissingEOF { position: usize },    // expected EOF, but there is more data
    UTF8Error { start_position: usize, end_position: usize, err: std::str::Utf8Error },
    NonzeroPadding { start_position: usize, error_position: usize, value: u8 },
}

impl PrimitiveParseError {
    pub fn position(&self) -> usize {
        match self {
            PrimitiveParseError::UnexpectedEOF { position } => *position,
            PrimitiveParseError::MissingEOF { position } => *position,
            PrimitiveParseError::UTF8Error { start_position, err, .. } => start_position + err.valid_up_to(),
            PrimitiveParseError::NonzeroPadding { error_position, .. } => *error_position,
        }
    }

    pub fn to_msg(&self, input: &[u8]) -> String {
        match self {
            PrimitiveParseError::UnexpectedEOF { position } => format!("unexpected EOF at position {position}"),
            PrimitiveParseError::MissingEOF { position } => format!("trailing data after expected EOF at position {position}"),
            PrimitiveParseError::UTF8Error { start_position, end_position, err } => {
                let data = String::from_utf8_lossy(&input[(*start_position as usize)..(*end_position as usize - 1)]);
                format!("UTF-8 error for string \"{}\" at position {start_position}: {err:?}", data.escape_debug())
            }
            PrimitiveParseError::NonzeroPadding {
                start_position,
                error_position,
                value,
            } => format!(
                "cstring padding value {} (0x{:02x}) is nonzero at position {} (string starts at {})",
                value, value, error_position, start_position
            ),
        }
    }
}

pub fn parse_u8(ctx: ParseContext) -> Result<(u8, ParseContext), PrimitiveParseError> {
    let ParseContext { strict, input, mut position } = ctx;
    let value = input.get(position).cloned().ok_or(PrimitiveParseError::UnexpectedEOF { position })?;
    position += 1; // can never overflow because position < input.len() before the read
    Ok((value, ParseContext { strict, input, position }))
}

pub fn parse_eof(ctx: ParseContext) -> Result<(), PrimitiveParseError> {
    match parse_u8(ctx.clone()) {
        Err(PrimitiveParseError::UnexpectedEOF { .. }) => Ok(()),
        Ok(_) => Err(PrimitiveParseError::MissingEOF { position: ctx.position }),
        Err(_) => unreachable!(),
    }
}

fn parse_2_u8(ctx: ParseContext) -> Result<([u8; 2], ParseContext), PrimitiveParseError> {
    let mut ctx = ctx.clone();
    let mut arr: [u8; 2] = [0, 0];
    for x in arr.iter_mut() {
        (*x, ctx) = parse_u8(ctx)?;
    }
    Ok((arr, ctx))
}

fn parse_4_u8(ctx: ParseContext) -> Result<([u8; 4], ParseContext), PrimitiveParseError> {
    let mut ctx = ctx.clone();
    let mut arr: [u8; 4] = [0, 0, 0, 0];
    for x in arr.iter_mut() {
        (*x, ctx) = parse_u8(ctx)?;
    }
    Ok((arr, ctx))
}

pub fn parse_le_u16(ctx: ParseContext) -> Result<(u16, ParseContext), PrimitiveParseError> {
    let (data, ctx) = parse_2_u8(ctx)?;
    Ok((u16::from_le_bytes(data), ctx))
}

pub fn parse_le_u32(ctx: ParseContext) -> Result<(u32, ParseContext), PrimitiveParseError> {
    let (data, ctx) = parse_4_u8(ctx)?;
    Ok((u32::from_le_bytes(data), ctx))
}

pub fn parse_le_i16(ctx: ParseContext) -> Result<(i16, ParseContext), PrimitiveParseError> {
    let (data, ctx) = parse_2_u8(ctx)?;
    Ok((i16::from_le_bytes(data), ctx))
}

pub fn parse_le_i32(ctx: ParseContext) -> Result<(i32, ParseContext), PrimitiveParseError> {
    let (data, ctx) = parse_4_u8(ctx)?;
    Ok((i32::from_le_bytes(data), ctx))
}

pub fn parse_le_f32(ctx: ParseContext) -> Result<(f32, ParseContext), PrimitiveParseError> {
    let (data, ctx) = parse_4_u8(ctx)?;
    Ok((f32::from_le_bytes(data), ctx))
}

pub fn parse_bytes(ctx: ParseContext, n: usize) -> Result<(&[u8], ParseContext), PrimitiveParseError> {
    let start_ctx = ctx.clone();
    let mut end_ctx = ctx.clone();
    for _ in 0..n {
        let (_, ctx) = parse_u8(end_ctx)?;
        end_ctx = ctx;
    }
    let start_position = start_ctx.position;
    let end_position = end_ctx.position;
    let data = &ctx.input[start_position..end_position];
    Ok((data, end_ctx))
}

pub fn parse_utf8cstring(ctx: ParseContext) -> Result<(std::borrow::Cow<'_, str>, ParseContext), PrimitiveParseError> {
    // TODO: test
    let start_ctx = ctx.clone();
    let mut end_ctx = ctx.clone();
    loop {
        let (x, ctx) = parse_u8(end_ctx)?;
        end_ctx = ctx;
        if x == 0 {
            break;
        }
    }
    let start_position = start_ctx.position;
    let end_position = end_ctx.position;
    let s = &ctx.input[start_position..(end_position - 1)];

    match std::str::from_utf8(s) {
        Ok(s) => Ok((std::borrow::Cow::Borrowed(s), end_ctx)),
        Err(e) => {
            if ctx.strict {
                Err(PrimitiveParseError::UTF8Error { start_position, end_position, err: e })
            } else {
                Ok((String::from_utf8_lossy(s), end_ctx))
            }
        }
    }
}

pub fn parse_utf8cstring_padded(ctx: ParseContext, chunk_size: usize) -> Result<(std::borrow::Cow<'_, str>, ParseContext), PrimitiveParseError> {
    let start_ctx = ctx.clone();
    let (s, mut end_ctx) = parse_utf8cstring(ctx)?;
    while (end_ctx.position() - start_ctx.position()) % chunk_size != 0 {
        let (x, ctx) = parse_u8(end_ctx)?;
        if ctx.strict && x != 0 {
            return Err(PrimitiveParseError::NonzeroPadding {
                start_position: start_ctx.position,
                error_position: ctx.position(),
                value: x,
            });
        }
        end_ctx = ctx;
    }
    Ok((s, end_ctx))
}

pub fn serialize_utf8cstring<W>(out: &mut W, s: &str) -> Result<(), std::io::Error>
where
    W: std::io::Write,
{
    _ = out.write(s.split(char::from_u32(0).unwrap()).next().expect("There should be at least one slice").as_bytes())?;
    // TODO: check that there's exactly one slice?
    _ = out.write(&[0])?;
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    pub fn test_parse() {
        let input = "abcd";
        let ctx = super::ParseContext::start_of(input.as_bytes());
        assert_eq!(ctx.position, 0);
        let (a, ctx) = super::parse_u8(ctx).unwrap();
        assert_eq!(ctx.position, 1);
        assert_eq!(a, 97);
        let (b, ctx) = super::parse_u8(ctx).unwrap();
        assert_eq!(ctx.position, 2);
        assert_eq!(b, 98);
        let (c, ctx) = super::parse_u8(ctx).unwrap();
        assert_eq!(ctx.position, 3);
        assert_eq!(c, 99);
        let (d, ctx) = super::parse_u8(ctx).unwrap();
        assert_eq!(ctx.position, 4);
        assert_eq!(d, 100);
        assert!(super::parse_u8(ctx.clone()).is_err());
        assert_eq!(super::parse_eof(ctx.clone()), Ok(()));
    }
}
