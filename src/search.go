// Copyright (C) Brandon Waite 2020  - All Rights Reserved
// Unauthorized copying of this file, via any medium, is strictly prohibited
// Proprietary
// Updated by Brandon Waite, May 28 2020

package main

import (
	"context"
	"database/sql"
	"fmt"
	"math"
	"strings"
	"time"

	"github.com/mattn/go-tty"
)

func cursorTo(n int) []byte {
	s := fmt.Sprintf("\033[%dG", n)
	return []byte(s)
}

const ESC = 033 // 27
var cursorSave = []byte{ESC, '7'}
var cursorRestore = []byte{ESC, '8'}
var cursorUp = []byte{ESC, '[', '1', 'F'}
var cursorDown = []byte{ESC, '[', '1', 'B'}
var left = []byte{ESC, '[', '1', 'D'}
var clearRight = []byte{ESC, '[', '0', 'K'}
var clearLeft = []byte{ESC, '[', '1', 'K'}
var clearLine = []byte{ESC, '[', '2', 'K'}
var del = []byte{ESC, '[', '1', 'P'}
var delLine = []byte{ESC, '[', '1', 'M'}
var next = []byte{ESC, '[', '1', 'E'}
var upButton = []byte{ESC, '[', 'A'}
var downButton = []byte{ESC, '[', 'B'}
var rightButton = []byte{ESC, '[', 'C'}
var leftButton = []byte{ESC, '[', 'D'}

type Direction int

const (
	SearchOlder Direction = iota
	SearchNewer
)

type Searcher func(context.Context, string, Direction, *int) ([]string, error)

func HistoricalSearch(ctx context.Context, searcher Searcher, args []string) error {
	text := strings.Join(args, " ")

	results, err := searcher(ctx, text, SearchOlder, nil)
	if err != nil {
		return err
	}

	for i := len(results) - 1; i >= 0; i-- {
		fmt.Println(results[i])
	}

	return nil
}

var debugSleepy = "debug:slow-mode"

const ARROW_UP = 'A'
const ARROW_DOWN = 'B'
const ARROW_RIGHT = 'C'
const ARROW_LEFT = 'D'

func Interactive(ctx context.Context, searcher Searcher) error {
	// TODO signal catch (pretty sure it's messing with git patch mode really hard)
	// TODO cursor resume_at_buffer tracking
	// TODO clear screen
	tty, err := tty.Open()
	if err != nil {
		return err
	}
	defer tty.Close()

	// restore, err := tty.Raw()
	// defer restore()

	out := tty.Output()
	write := func(bufs ...[]byte) error {
		for _, b := range bufs {
			if sleepy, ok := ctx.Value(debugSleepy).(time.Duration); sleepy > 0 && ok {
				time.Sleep(sleepy)
			}

			if _, err := out.Write(b); err != nil {
				return err
			}
		}
		return nil
	}

	tty.Size()
	write([]byte{033, '[', '6', 'n'})
	r, err := tty.ReadRune()
	debug.WriteString(fmt.Sprintf("%d %+v \t", r, err))
	for tty.Buffered() {
		r, err := tty.ReadRune()
		debug.WriteString(fmt.Sprintf("%d %+v \t", r, err))
	}
	debug.WriteString(" -end- \n")

	write(cursorSave)
	preview := "] "
	prompt := "search: "

	lastSearchPosition := new(int)
	*lastSearchPosition = math.MaxInt64
	direction := SearchOlder

	query := []byte{}
	result := string(query)
	// TODO maxlen 80 or tty width
	// TODO underscore matching patch of string
	for {
		write(cursorRestore, cursorSave, clearRight)
		out.WriteString(preview + result + "\n" + prompt + string(query))

		r, err := tty.ReadRune()
		if err != nil {
			return err
		}

		if r < 32 {
			if !tty.Buffered() {
				goto end
			}

			debug.WriteString(fmt.Sprintf("KeyCode[1]<32 = %d \n", r))
			if r != 27 {
				goto end
			}

			r, err = tty.ReadRune()
			if err != nil {
				return err
			}

			debug.WriteString(fmt.Sprintf("KeyCode[1]<32 = %d \n", r))
			if r != '[' {
				goto end
			}

			r, err = tty.ReadRune()
			if err != nil {
				return err
			}
			debug.WriteString(fmt.Sprintf("KeyCode[2]<32 = %d \n", r))

			switch r {
			case ARROW_UP:
				*lastSearchPosition -= 1
				debug.WriteString(fmt.Sprintf("SearchOlder: %d \n", *lastSearchPosition))
				direction = SearchOlder
			case ARROW_DOWN:
				*lastSearchPosition += 1
				debug.WriteString(fmt.Sprintf("SearchNewer: %d \n", *lastSearchPosition))
				direction = SearchNewer
			default:
				goto end
			}
		} else if r == 127 {
			if len(query) == 0 {
				continue
			}

			// TODO https://www.vt100.net/docs/vt510-rm/DCH.html
			query = query[:len(query)-1]
			pos := len(preview) + len(query)
			write(left, del, next, cursorTo(pos), del)
		} else {
			// handle key event
			query = append(query, byte(r))
		}

		debug.WriteString(fmt.Sprintf("Searching: %s %d %d \n", string(query), direction, *lastSearchPosition))
		results, err := searcher(ctx, string(query), direction, lastSearchPosition)
		if err != nil {
			return err
		}
		if len(results) == 0 {
			debug.WriteString(string(query) + " :: len(results) = 0 :: " + err.Error())
			results = []string{"<err state -- check debug.log>"}
		}
		result = results[0]

		write(delLine)
	}

end:
	write(delLine, cursorUp, cursorRestore, clearRight)

	fmt.Println(result)
	return nil
}

// var match = `command MATCH 'NEAR(?, 10)'`
const match = `command LIKE '%' || ? || '%'`

func ListRecent(db *sql.DB) Searcher {
	// MAYBE care about direction and position
	return func(ctx context.Context, text string, _ Direction, _ *int) ([]string, error) {
		rows, err := db.QueryContext(ctx, "SELECT rowid, command FROM history WHERE "+match+" ORDER BY rowid DESC LIMIT 20", text)
		if err != nil {
			return nil, err
		}
		defer rows.Close()

		results := make([]string, 0, 20)
		for rows.Next() {
			var id int
			var r string
			if err := rows.Scan(&id, &r); err != nil {
				return nil, err
			}
			results = append(results, fmt.Sprintf("%4d  %s", id, r))
		}

		return results, nil
	}
}

func FindMatch(db *sql.DB) Searcher {
	return func(ctx context.Context, text string, direction Direction, lastOid *int) ([]string, error) {
		if lastOid == nil {
			lastOid = new(int)
		}

		if text == "" {
			return []string{""}, nil
		}

		var rows *sql.Rows
		var err error

		switch direction {
		case SearchOlder:
			rows, err = db.QueryContext(ctx, "SELECT oid, command FROM history WHERE "+match+" AND oid <= ? ORDER BY oid DESC LIMIT 1", text, *lastOid)
		case SearchNewer:
			rows, err = db.QueryContext(ctx, "SELECT oid, command FROM history WHERE "+match+" AND oid >= ? ORDER BY oid ASC LIMIT 1", text, *lastOid)
		default:
			err = fmt.Errorf("Unknown direction enum: %d", direction)
		}
		if err != nil {
			return nil, err
		}

		if !rows.Next() {
			return []string{"<no search found>"}, nil
		}

		var s string
		if err := rows.Scan(lastOid, &s); err != nil {
			return nil, err
		}
		return []string{s}, err
	}
}

var TTYSleep = ""
var sleep time.Duration

func init() {
	sleep, _ = time.ParseDuration(TTYSleep)
}

func Search(ctx context.Context, args []string) error {
	if err := initDebug(); err != nil {
		return err
	}
	defer debug.Close()

	// enable slow cursor movements
	if TTYSleep != "" {
		ctx = context.WithValue(ctx, debugSleepy, sleep)
	}

	dbpath, err := dir.Filepath(data, indexDatabase)
	if err != nil {
		return err
	}

	db, err := sql.Open("sqlite3", dbpath)
	if err != nil {
		return err
	}
	defer db.Close()

	if len(args) > 0 && args[0] == "--interactive" {
		return Interactive(ctx, FindMatch(db))
	}

	return HistoricalSearch(ctx, ListRecent(db), args)
}
