package db

import (
	"context"

	"github.com/generativelabs/btcserver/internal/db/ent"
	"github.com/generativelabs/btcserver/internal/db/ent/stake"
)

func (c *Backend) CreateStake(
	staker string,
	txID string,
	start int64,
	duration int64,
	amount int64,
	rewardReceiver string,
) error {
	_, err := c.dbClient.Stake.Create().
		SetStaker(staker).
		SetTx(txID).
		SetStart(start).
		SetDuration(duration).
		SetAmount(amount).
		SetRewardReceiver(rewardReceiver).
		Save(context.Background())

	return err
}

func (c *Backend) QueryStakesByStaker(
	staker string,
) ([]*ent.Stake, error) {
	return c.dbClient.Stake.Query().Where(stake.StakerEQ(staker)).All(context.Background())
}

func (c *Backend) QueryNotEndStatesTx(limit int) ([]string, error) {
	return c.dbClient.Stake.Query().Where(stake.EndEQ(false)).
		Limit(limit).Select(stake.FieldTx).Strings(context.Background())
}
