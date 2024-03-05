package db

import (
	"fmt"

	"github.com/generativelabs/btcserver/internal/db/ent"
	_ "github.com/go-sql-driver/mysql" // mysql driver
)

type Config struct {
	User     string `mapstructure:"user"`
	Host     string `mapstructure:"host"`
	Database string `mapstructure:"database	"`
	Password string `mapstructure:"password"`
}

type Backend struct {
	dbClient *ent.Client
}

func CreateBackend(config Config) (*Backend, error) {
	client, err := ent.Open("mysql", fmt.Sprintf("%s:%s@tcp(%s)/%s?parseTime=True",
		config.User, config.Password, config.Host, config.Database))
	if err != nil {
		return nil, err
	}

	dbClient := Backend{
		dbClient: client,
	}

	return &dbClient, nil
}
