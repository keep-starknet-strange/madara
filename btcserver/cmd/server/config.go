package server

import "github.com/generativelabs/btcserver/internal/db"

type Config struct {
	Btc struct {
		// NetworkName defines the bitcoin network name
		NetworkName string `mapstructure:"network-name"`
		// RPCHost defines the bitcoin rpc host
		RPCHost string `mapstructure:"rpc-host"`
		// RPCUser defines the bitcoin rpc user
		RPCUser string `mapstructure:"rpc-user"`
		// RPCPass defines the bitcoin rpc password
		RPCPass string `mapstructure:"rpc-pass"`
		// DisableTLS defines the bitcoin whether tls is required
		DisableTLS bool `mapstructure:"disable-tls"`
	} `mapstructure:"btc"`

	Chakra struct {
		URL        string `mapstructure:"http-url"`
		ChainID    string `mapstructure:"chain-id"`
		PrivateKey string `mapstructure:"private-key"`
	} `mapstructure:"chakra"`

	Mysql db.Config `mapstructure:"mysql"`

	ServicePort int `mapstructure:"service-port"`
}
