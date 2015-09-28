package ec2

import (
	"fmt"
	"net"
	"time"

	"github.com/coreos/coreos-metadata/src/config"
	"github.com/coreos/coreos-metadata/src/retry"
)

func FetchMetadata() (config.Metadata, error) {
	public, err := fetchIP("http://169.254.169.254/2009-04-04/meta-data/public-ipv4")
	if err != nil {
		return config.Metadata{}, err
	}
	local, err := fetchIP("http://169.254.169.254/2009-04-04/meta-data/local-ipv4")
	if err != nil {
		return config.Metadata{}, err
	}
	hostname, err := fetchString("http://169.254.169.254/2009-04-04/meta-data/hostname")
	if err != nil {
		return config.Metadata{}, err
	}

	return config.Metadata{
		PublicIPv4: public,
		LocalIPv4:  local,
		Hostname:   hostname,
	}, nil
}

func fetchString(url string) (string, error) {
	body, err := retry.Client{
		InitialBackoff: time.Second,
		MaxBackoff:     time.Second * 5,
		MaxAttempts:    10,
	}.Get(url)
	return string(body), err
}

func fetchIP(url string) (net.IP, error) {
	str, err := fetchString(url)
	if err != nil {
		return nil, err
	}
	if ip := net.ParseIP(str); ip != nil {
		return ip, nil
	} else {
		return nil, fmt.Errorf("couldn't parse %q as IP address", str)
	}
}
