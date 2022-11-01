# StackrankDice

This is re-implementation of [DiceWars](https://www.gamedesign.jp/games/dicewars/) game by [GAMEDESIGN.jp](https://www.gamedesign.jp/)

https://user-images.githubusercontent.com/97428129/199223618-212ead8f-30ae-443f-aa2f-4abb1f2397cf.mp4

## How to play

On a hexagon board each player starts with a number of regions. Each region has a number of dice. The goal is to conquer all regions of the opponent.

Battle mechanics is simple: each player rolls a number of dice equal to the number of dice in the region. The player with the highest number of dice wins. In case of a tie, the attacker loses.

Conquered regions are added to the attacker's stack. The attacker can choose to move some of the dice to the conquered region. The number of dice in the conquered region cannot be less than 1.

## Implementation

This is a re-implementation with [Bevy](https://bevyengine.org/) engine on Rust langaage. The original game was written in Flash.

## Acknowledgements

- [bevy-hex-example](https://github.com/Quantumplation/bevy-hex-example) by [Pi Lanningham](https://github.com/Quantumplation/bevy-hex-example) — for general hex grid implementation. Code has no license on github.
