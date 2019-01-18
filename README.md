# Wizard's Castle, ncurses version

Complete and utter WIP.

This is intended to be an ncurses front end on the [Wizard's Castle
backend](https://github.com/beejjorgensen/Wizards-Castle-Rust) I wrote earlier.
(The backend also comes with a non-curses classic version of the game.)

[More Wizard's Castle info](https://github.com/beejjorgensen/Wizards-Castle-Info)

## TODO

* Room effects
  * Warp
  * Sinkhole
  * Gold pickup
  * Flare pickup

* Actions
  * Normalize arrow keys to NSWE
  * Drink
  * Gaze
  * Flare
  * Down
  * Up
  * Lamp
  * Teleport
  * Open book
  * Open chest
  * Quit
  * Help

* Combat
  * Attack
  * Retreat

* Trading
  * Sell treasures
  * Buy armor
  * Buy weapons
  * Buy potions
  * Buy lamp
  * Buy flares

* Curses
  * Lethargy
  * The Leech
  * Forgetfulness

* Game over screen

* `refresh()` instead of `wrefresh()` to cure some repaint ills?
