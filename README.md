<p align="center">
  <h1 align="center">Game Of Life</h1>
</p>

- [About](#about)
- [Attribution / Credits](#attribution--credits)
- [Screenshots / Demo](#screenshots--demo)
- [Install](#install)
- [Authors](#authors)


## About
A Simulation of Conway's Game of Life extrapolated to 3 Dimensions. Uses the bevy game engine to render both the ui and actual simulation. As there's no clearly defined ruleset for the 3d version, multiple ones are used and can be chosen, each displaying a different development, from randomly generated starting points. There exist also different starting rules with a predefined starting state/form to observe different shapes/results - which follow specific interesting patterns (intricate shapes, parallel simulations & more)

Configurable Rulesets, probability and Dimension size
Can be paused ,reset ,(un)focused during simulation, along with reactive speed adjustments.
When Paused - camera movable to inspect cubes (with WASD key), when reset camera teleported back to original position

## Attribution / Credits
The reference for rules to implement the (clearly labeled in the source code) functions/rules that end up dissolving into predictable or stable structures has been taken from the paper by Carter Bays "Candidates for the game of life in three dimensions" (1987), full paper can be found <a href="https://wpmedia.wolfram.com/sites/13/2018/02/01-3-1.pdf" target="_blank" rel="noreferrer" alt="link" >here</a>

## Screenshots / Demo

![App Screenshot](images/menu.jpg)
<p align="center">Menu - all parameters configurable, speed adjustable mid simulation</p>
<hr>

![App Screenshot](images/initial.jpg)
<p align="center">Initial state at 5% spawn prob.</p>
<hr>

![App Screenshot](images/balanced.jpg)
<p align="center">Balanced Rule with 5% spawn prob.</p>
<hr>

![App Screenshot](images/dense.jpg)
<p align="center">Dense Rule with 10% spawn prob.</p>
<hr>

![App Screenshot](images/sparse.jpg)
<p align="center">Sparse Rule with 5% spawn prob.</p>
<hr>

![App Screenshot](images/nodeath.jpg)
<p align="center">No Death Rule</p>
<hr>

![App Screenshot](images/tunnel.jpg)
<p align="center">Tunnel Like with 25% spawn prob</p>
<hr>

![App Screenshot](images/closest.jpg)
<p align="center">At 25% spawn. prob Rule (5,7,6,6) is the closest known rule for mimicking the results of the classic 2d variation of the experiment - detailed desc. in rules.rs</p>

![App Screenshot](images/barrier1.jpg)
<p align="center">Same Rule - but fixed placed stable structure , creates stable parallel barriers between which mini simulations of a 3d conway universe can run simultaneously</p>

![App Screenshot](images/barrier3.jpg)
<p align="center">Mini stable structures converged from randomly generated cubes between parallel barriers</p>

![App Screenshot](images/barrier2.jpg)
<p align="center">Empty side view</p>
<hr>

![App Screenshot](images/rule4555.jpg)
<p align="center">At 25% spawn. prob & R = (4,5,5,5) is a rule that produces semi-stable small structures that display symmetry</p>

![App Screenshot](images/rule4555_stable.jpg)
<p align="center">Same Rule - but fixed handplaced stable structures that would be almost impossible to find through randomness</p>
<hr>

![App Screenshot](images/misc1.jpg)
<p align="center">Misc. Unstable structures found - 1</p>
<br>

![App Screenshot](images/misc2.jpg)
<p align="center">Misc. Unstable structures found - 2</p>
<hr>

## Install
clone repo and build (need rust compiler installed)

## Authors

* ThatLukaszGuy

