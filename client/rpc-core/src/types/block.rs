#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BlockId {
    Num(u64),
    Latest,
    Pending,
}

impl Default for BlockId {
    fn default() -> Self {
        BlockId::Latest
    }
}

impl<'a> Deserialize<'a> for BlockId {
    fn deserialize<D>(deserializer: D) -> Result<BlockId, D::Error>
    where
        D: Deserializer<'a>,
    {
        deserializer.deserialize_any(BlockIdVisitor)
    }
}

impl Serialize for BlockId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Block::num(ref x) => serializer.serialize_str(&format!{"0x{:x}", x}),
            BlockId::Latest => serializer.serialize_str("latest"),
            BlockId::Pending => serializer.serialize_str("pending"),
        }
    }
}

struct BlockIdVisitor;

impl<'a> Visitor<'a> for BlockIdVisitor {
    type Value = BlockId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a block number or 'latest' or 'pending'"
        )
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'a>,
    {
        let mut block_number = None::<u64>;

        loop {
            let key_str: Option<String> = visitor.next_key()?;
            
            match key_str {
                Some(key) => match key.as_str() {
                    "blockNumber" => {
                        let value: String = visitor.next_value()?;
                        if let Some(stripped) = value.strip_prefix("0x") {
                            let number = u64::from_str_radix(stripped, 16).map_err(|e| {
                                Error::custom(format!("Invalid block number: {}", e))
                            })?;

                            block_number = Some(number);
                            break;
                        } else {
                            let number = u64::from_str_radix(value, 10).map_err(|e| {
                                Error::custom(format!("Invalid block number: {}", e))
                            })?;

                            block_number = Some(number);
                            break;
                        }
                    }
                },
                None => break,
            }
        }

        if let Some(number) = block_number {
			return Ok(BlockId::Num(number));
		}

        Err(Error::custom("Invalid input"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        match value {
            "latest" => Ok(BlockId::Latest),
            "pending" => Ok(BlockId::Pending),
            _ => value.parse::<u64>().map(BlockId::Num).map_err(|_|{
                Error::custom("Invalid block id: -".to_string())
            }),
        }
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
	where
		E: Error,
	{
		self.visit_str(value.as_ref())
	}

	fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
	where
		E: Error,
	{
		Ok(BlockId::Num(value))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

    fn match_block_id(block_id: BlockId) -> Option<u64> {
		match block_id {
			BlockId::Num(number) => Some(number),
			BlockId::Latest => Some(1000),
			BlockId::Pending => Some(1001),
			_ => None,
		}
	}

    #[test]
    fn block_id_deserialize() {
        let bn_dec: BlockId = serde_json::from_str(r#""42""#).unwrap();
        let bn_hex: BlockId = serde_json::from_str(r#""0x45""#).unwrap();

        assert_eq!(match_block_id(bn_dec).unwrap(), 42);
        assert_eq!(match_block_id(bn_hex).unwrap(), 69);
    }
}