package server

import (
	"github.com/generativelabs/btcserver/internal/api"
	"github.com/generativelabs/btcserver/internal/db"
	"github.com/rs/zerolog/log"
	"github.com/spf13/viper"
)

func init() {
	viper.SetConfigType("yaml")

	viper.SetConfigName("btc-server.yml")
	viper.AddConfigPath(".")

	if err := viper.ReadInConfig(); err != nil {
		log.Fatal().Msgf("Fatal error config file: %s ", err)
	}
}

func Run() {
	var config Config
	err := viper.Unmarshal(&config)
	if err != nil {
		log.Fatal().Msgf("❌ Fatal error decode config into struct: %s ", err)
	}

	backend, err := db.CreateBackend(config.Mysql)
	if err != nil {
		log.Fatal().Msgf("❌ Fatal error create db backend: %s ", err)
	}

	apiServer := api.New(backend)
	err = apiServer.Run(config.ServicePort)
	if err != nil {
		log.Fatal().Msgf("❌ Fatal error in api server: %s ", err)
	}
}
