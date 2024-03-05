package api

import (
	"fmt"
	"net/http"

	"github.com/generativelabs/btcserver/internal/db"
	"github.com/gin-gonic/gin"
)

type Server struct {
	backend *db.Backend
	engine  *gin.Engine
}

type Staker struct {
	Staker string `form:"staker"`
}

func New(backend *db.Backend) *Server {
	server := &Server{
		backend: backend,
	}

	r := gin.Default()
	r.GET("/stakes", server.GetStakesByStaker)

	return server
}

func (s Server) GetStakesByStaker(c *gin.Context) {
	var staker Staker
	err := c.Bind(&staker)
	if err != nil {
		c.JSON(http.StatusBadRequest, gin.H{
			"error": err.Error(),
		})
		return
	}

	stakes, err := s.backend.QueryStakesByStaker(staker.Staker)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, stakes)
}

func (s Server) Run(servicePort int) error {
	return s.engine.Run(fmt.Sprintf(":%d", servicePort))
}
