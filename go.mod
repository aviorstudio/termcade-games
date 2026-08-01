// Every aviorstudio game, in one module.
//
// Games depend on the sdk and nothing else — that is the whole contract, and
// sharing one module across all of them means they share one sdk version too,
// so a contract change is caught here for every game at once instead of drifting
// game by game.
module github.com/aviorstudio/termcade-games

go 1.26.2

require github.com/aviorstudio/termcade/sdk v0.0.1
