//go:build wasip1

// The brickough wasm guest: packages the game as an installable .tcade.
// Nothing is compiled into the arcade — the built package is vendored into
// its starter pack and is what players install from the marketplace.
package main

import (
	"github.com/aviorstudio/termcade-games/brickough"
	"github.com/aviorstudio/termcade/sdk/tcgame"
)

func init() { tcgame.Register(brickough.New) }

func main() {} // never runs; the module is a wasip1 reactor
