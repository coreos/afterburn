package config

import (
	"net"
)

type Metadata struct {
	PublicIPv4 net.IP
	LocalIPv4  net.IP
	Hostname   string
}
