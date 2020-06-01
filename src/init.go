// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

package main

import (
	"bufio"
	"context"
	"database/sql"
	"errors"
	"fmt"
	"os"
	"path"
	"strconv"
	"strings"
	"time"

	sqlite3 "github.com/mattn/go-sqlite3"
)

// MAYBE should become init-common / init-core?
func Init(ctx context.Context) error {
	alreadyExists, err := dir.Ensure()
	if err != nil {
		return err
	}

	if err := initDebug(); err != nil {
		return err
	}
	defer debug.Close()

	dbpath, err := dir.Filepath(data, indexDatabase)
	if err != nil {
		return err
	}

	db, err := sql.Open("sqlite3", dbpath)
	if err != nil {
		return err
	}
	defer db.Close()

	schema, err := Asset("src/schema.sql")
	if err != nil {
		return err
	}

	if _, err := db.ExecContext(ctx, string(schema)); err != nil {
		sqliteErr, ok := err.(sqlite3.Error)
		if !ok {
			return err
		}

		if !strings.Contains(sqliteErr.Error(), "already exists") {
			return err
		}

		// fmt.Println(sqliteErr.Code, sqliteErr.ExtendedCode, sqliteErr.SystemErrno, sqliteErr.Error())
	}

	var initScript []byte
	shell := path.Base(os.Getenv("SHELL"))
	switch shell {
	case "zsh":
		if !alreadyExists {
			// TODO #001# import only if init is ran with flag
			if err := importZshHistory(ctx, db); err != nil {
				return err
			}
		}

		initScript, err = Asset("src/init.zsh")
		if err != nil {
			return err
		}
	case "fish":
		if !alreadyExists {
			// TODO #001#
			// if err := importFishHistory(ctx, db); err != nil {
			// 	return err
			// }
		}
		initScript, err = Asset("src/init.fish")
		if err != nil {
			return err
		}
	default:
		return errors.New(shell + " is not a currently supported shell")
	}

	fmt.Println(string(initScript))
	return nil
}

func importZshHistory(ctx context.Context, db *sql.DB) error {
	home, err := os.UserHomeDir()
	if err != nil {
		return err
	}

	history, err := os.Open(path.Join(home, ".zsh_history"))
	if err != nil {
		if !os.IsNotExist(err) {
			return err
		}

		debug.WriteString("# History file was not found on init attempt \n")
	}

	if history != nil {
		scanner := bufio.NewScanner(history)
		for scanner.Scan() {
			_, err := db.ExecContext(ctx, "INSERT INTO history(command, timestamp) VALUES (?, ?)", scanner.Text(), now())
			if err != nil {
				return err
			}
		}
	}

	return nil
}

func importFishHistory(ctx context.Context, db *sql.DB) error {
	type FishCommand struct {
		Command string `yaml:"cmd"`
		When    int64  `yaml:"when"`
	}

	home, err := os.UserHomeDir()
	if err != nil {
		return err
	}

	history, err := os.Open(path.Join(home, ".local", "share", "fish", "fish_history"))
	if err != nil {
		return err
	}

	// not valid yaml apparently :/
	// if err := yaml.NewDecoder(history).Decode(commands); err != nil {
	// 	return err
	// }

	commands := make([]*FishCommand, 0)
	scanner := bufio.NewScanner(history)
	var current *FishCommand
	for scanner.Scan() {
		if strings.HasPrefix(scanner.Text(), "- cmd: ") {
			cmd := strings.Split(scanner.Text(), "- cmd:")[1]

			current = &FishCommand{Command: cmd}
			commands = append(commands, current)
		} else {
			when := strings.Split(scanner.Text(), "when: ")[1]
			current.When, err = strconv.ParseInt(strings.TrimSpace(when), 10, 0)
			if err != nil {
				return err
			}
		}
	}

	for _, command := range commands {
		_, err := db.ExecContext(ctx, "INSERT INTO history(command, timestamp) VALUES (?, ?)", command.Command, time.Unix(command.When, 0))
		if err != nil {
			return err
		}
	}

	return nil
}
