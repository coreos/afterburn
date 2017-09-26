package exec

import (
	"bytes"
	"fmt"
	"os/exec"
	"syscall"
)

// Exec will run the given command and arguments, sending in the stdin string
// over stdin, and will return either the stdout, stderr, and exit code from the
// command, or an error.
func Exec(stdin string, cmd string, args ...string) (string, string, int, error) {
	var stdout, stderr bytes.Buffer
	command := exec.Command(cmd, args...)
	command.Stdout = &stdout
	command.Stderr = &stderr
	if stdin != "" {
		command.Stdin = bytes.NewBufferString(stdin)
	}
	err := command.Run()
	if exitErr, ok := err.(*exec.ExitError); ok {
		code := exitErr.Sys().(syscall.WaitStatus).ExitStatus()
		return stdout.String(), stderr.String(), code, nil
	}
	if err != nil {
		return "", "", -1, err
	}
	return stdout.String(), stderr.String(), 0, nil
}

// ExecNoStdin behaves the same as Exec, but doesn't pass in anything over stdin
func ExecNoStdin(cmd string, args ...string) (string, string, int, error) {
	return Exec("", cmd, args...)
}

// ExecExpectZero will run the given command and arguments, and will return an
// error if either there is an issue running the command or if the command
// doesn't exit with code 0.
func ExecExpectZero(cmd string, args ...string) error {
	stdout, stderr, code, err := Exec("", cmd, args...)
	if err != nil {
		return err
	}
	if code != 0 {
		return fmt.Errorf("%s exited with %d, stdout:%q stderr:%q", cmd, code, stdout, stderr)
	}
	return nil
}
