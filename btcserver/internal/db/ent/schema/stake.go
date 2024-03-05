package schema

import (
	"entgo.io/ent"
	"entgo.io/ent/schema/field"
	"entgo.io/ent/schema/index"
)

type Stake struct {
	ent.Schema
}

func (Stake) Fields() []ent.Field {
	return []ent.Field{
		field.String("staker").MaxLen(90),      // btc address
		field.String("tx").MaxLen(66).Unique(), // btc transaction id len is 64byte, and len of prefix "0x" is 2byte.
		field.Int64("start"),                   // btc stake start timestamp
		field.Int64("duration"),                // btc stake end timestamp
		field.Int64("amount"),                  // btc stake amount
		field.String("receiver").MaxLen(66),    // starknet address to receive reward. length is 64byte, and length of prefix "0x" is 2byte.
		field.Bool("finalized"),                // btc transaction weather finalized.
		field.Bool("end").Default(false),       // stake epoch is over.
		field.String("btc_sig"),                // signature for btc transaction.
		field.String("receiver_sig"),           // signature for receiver address.
	}
}

func (Stake) Indexes() []ent.Index {
	return []ent.Index{
		index.Fields("staker", "tx"),
		index.Fields("end", "tx"),
	}
}
