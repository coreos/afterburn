// Copyright 2015 CoreOS, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package retry

import (
	"fmt"
	"io/ioutil"
	"net/http"
	"time"
)

type Client struct {
	InitialBackoff time.Duration
	MaxBackoff     time.Duration
	MaxAttempts    int
	Header         http.Header
}

func (c Client) Get(url string) ([]byte, error) {
	delay := c.InitialBackoff
	for attempt := 1; attempt <= c.MaxAttempts; attempt++ {
		fmt.Printf("fetching %q: attempt #%d\n", url, attempt)

		request, err := http.NewRequest("GET", url, nil)
		if err != nil {
			return nil, err
		}

		request.Header = c.Header

		if response, err := (&http.Client{}).Do(request); err != nil {
			fmt.Printf("failed to fetch: %v\n", err)
		} else if response.StatusCode == http.StatusNotFound {
			return nil, nil
		} else if response.StatusCode != http.StatusOK {
			fmt.Printf("failed to fetch: %s\n", http.StatusText(response.StatusCode))
		} else {
			defer response.Body.Close()
			return ioutil.ReadAll(response.Body)
		}

		time.Sleep(delay)
		delay *= 2
		if delay > c.MaxBackoff {
			delay = c.MaxBackoff
		}
	}

	return nil, fmt.Errorf("timed out while fetching %q", url)
}

func (c Client) Getf(format string, a ...interface{}) ([]byte, error) {
	return c.Get(fmt.Sprintf(format, a...))
}
