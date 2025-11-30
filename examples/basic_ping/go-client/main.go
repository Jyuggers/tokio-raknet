package main

import (
	"bytes"
	"fmt"
	"log/slog"
	"os"

	"github.com/sandertv/go-raknet"
)

func main() {
	var dialer raknet.Dialer
	dialer.ErrorLog = slog.New(slog.NewTextHandler(os.Stderr, nil))
	fmt.Println("[go-client] dialing 127.0.0.1:19135 ...")
	conn, err := dialer.Dial("127.0.0.1:19135")
	if err != nil {
		fmt.Printf("[go-client] dial error: %v\n", err)
		panic(err)
	}
	defer conn.Close()
	fmt.Println("[go-client] successfully connected.")

	fmt.Println("[go-client] writing payload: hello server")
	// 0x80 is the user packet ID we agreed upon
	payload := append([]byte{0x80}, []byte("hello server")...)
	if _, err := conn.Write(payload); err != nil {
		fmt.Printf("[go-client] write error: %v\n", err)
		return
	}

	b := make([]byte, 1024)

	fmt.Println("[go-client] waiting to read reply ...")

	n, err := conn.Read(b)
	if err != nil {
		fmt.Printf("[go-client] read error: %v\n", err)
		return
	}
	fmt.Printf("[go-client] read %d bytes\n", n)

	id := b[0]

	fmt.Printf("[go-client] received user packet id: 0x%x\n", id)

	nullIndex := bytes.IndexByte(b, 0x00)

	if nullIndex == -1 {
		fmt.Println("No null terminator found.")
		return
	}

	// Create a Go string from the byte slice up to the null terminator
	goString := string(b[1:nullIndex])

	fmt.Printf("client received: %s\n", goString)
}
