package main

import (
	"bytes"
	"fmt"
	"log"
	"log/slog"
	"os"

	"github.com/sandertv/go-raknet"
)

func main() {
	var lc raknet.ListenConfig
	lc.ErrorLog = slog.New(slog.NewTextHandler(os.Stderr, nil))
	listener, err := lc.Listen("0.0.0.0:19135")
	if err != nil {
		_ = fmt.Errorf("Error on listener: %w", err)
		panic(err)
	}
	fmt.Println("listening on 0.0.0.0:19135")
	defer listener.Close()
	for {
		conn, err := listener.Accept()
		if err != nil {
			log.Printf("Error on accept: %v", err)
			continue
		}

		fmt.Println("Accepted a new con.")

		b := make([]byte, 1024)

		_, _ = conn.Read(b)

		id := b[0]

		fmt.Printf("Recieved user packet id: 0x%x\n", id)

		nullIndex := bytes.IndexByte(b, 0x00)

		if nullIndex == -1 {
			fmt.Println("No null terminator found.")
			return
		}

		// Create a Go string from the byte slice up to the null terminator
		goString := string(b[1:nullIndex])

		fmt.Printf("server received: %s\n", goString)
		fmt.Println("server replied with: hello world")

		// 0x80 is the user packet ID
		reply := append([]byte{0x80}, []byte("hello world")...)
		_, _ = conn.Write(reply)

		conn.Close()
	}
}
