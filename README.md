# L3/L3X IDE

The L3/L3X language is a esolang created for the CMIMC 2023 competition. Below is the spec for
L3/L3X given in the competition problem statement, adapted for markdown (but otherwise verbatim):

# Language Specification

## L3

L3 is a program that manipulates natural numbers. An L3 program consists of a grid where each square
contains a natural number between 1 and 30, and a direction (up, down, left, right). The input to
the program is a natural number M, which enters the program on the top-left-most square, traveling
downwards. M moves around the grid and is manipulated with these rules:

On a single step of the program, suppose that M is on a square with number A and direction X. 

* If M is moving in direction X, M is multiplied by the A, and moves to the square in direction X.
* If M is moving in any of the three other directions, if M is divisible by A, M is divided by A,
and moves to the square in direction X. If M is not divisible by A, M is unchanged, and moves to the
square in the direction opposite of X.

The program outputs when M moves out of the bottom-right-most
square downwards.

---

Actual L3 code is formatted in a comma delimited csv file. It is suggested that you use a spreadsheet tool to write your code, such as Google Sheets or LibreOffice Calc (Microsoft Excel may export some extra invisible metadata, so you may need to copy the text from the csv export into a new csv file). Make sure to export exactly the size of the grid you intended, so the location of the output matches with what the interpreter expects. Hint: To help coding, use color highlighting. 

Within each cell, you should write the number followed by the direction in any of the following forms: u/d/l/r, U/D/L/R, n/s/w/e, N/S/W/E. Additionally, you may add a “;” at the end to indicate a watch point. 

For readability, you may leave squares empty, and an error will occur if M moves onto an empty square. Similarly, if M moves out the grid not in the bottom-right-most square downwards, an error will occur. Assume reasonable constraints on the maximum number of steps a program may execute (currently <ins>20000</ins>) and the maximum size of the grid (currently 100x100). 

---

Example A - clear register. Input = 2<sup>x</sup>. Output = 1.

| | | |
|-|-|-|
| `1R` | `2L` | `1D` |

Example B - transfer register. Input = 
2<sup>x</sup>. Output = 3<sup>x</sup>.

| | |
|-|-|
| `1D` | `1L` |
| `1D` | `3U` |
| `1R` | `2U` |

## L3X

It is not hard to argue that L3 is Turing-Complete. However, it is very tedious to deal with streams of data in L3, and encoding streams of data is terribly inefficient. The language L3 extended (L3X) aims to solve this issue.

In L3X, multiple numbers may be manipulated by the program simultaneously. Besides containing natural numbers, a grid may contain a ‘%’, ’&’, or ’~’, still with some direction X. 

* On a ‘%’ square, M is duplicated, where one M travels in the direction of X, and one M travels in the opposite direction. 
* On a ’&’ square (with direction X), when a number M enters the block in direction X, it is stored in a FIFO queue unique to the square. When another number N enters the block in direction not X, the number N is multiplied with the first element of the queue, M, M is removed from the queue, and the product NM travels in direction X. If the queue is empty, an error is raised.
* Finally, on a ‘~’ square, the number M is set to 1 and continues in the direction of X. 

There will be a limit of at most 10 different numbers running actively in an L3X (not stored in the queue of any ‘&’ square). Two numbers may never appear on the same square (excluding those stored in the queue of any ‘&’ square), and an error will be raised if that happens.

---

In any L3 program the grid is indexed by (row, column). If the grid has height h and width w, input to the program enters the grid on square (0, 0) moving in direction (1, 0), and the output of the program exits the grid on square (h-1, w-1) moving in direction (1, 0). 

In a L3X program the square at (0, 1) must be a “&” square. Inputs to the program will be a single number N in the top-left-most square, and a sequence of numbers already stored in the “&” block. The program should output a sequence of numbers traveling downwards from 
(h-1, w-2), and finally a single number downwards from (h-1, w-1).

Example C - move single number from input stream to output stream (special case of task 9). Input =
2<sup>1</sup> and \[ 2<sup>x</sup> \]. Output = 2<sup>1</sup> and \[ 2<sup>x</sup> \].

| | | | |
|-|-|-|-|
| `~E` | `&S` | `1E` | `1S` |
| `1S` | `%W` | `~N` | `2S` |
| `1E` | `1E` | `1S` | `1S` |

## Addendum (not from original problem statement)

This section will discuss both features of the language not covered by the problem specification
(but present in the problem), as well as features that this l3x implementation adds. 

* The "&" square queue does not quite allow multiple numbers on itself. If, for example, two
  numbers, one entering the queue and one popping from it, entered on the same tick, this would be
  counted as a collision.
  * This implementation optionally allows two numbers to enter the queue in this way without
    colliding. In this case, if the queue is empty, the result is as if the number had entered the
    queue, and immediately after is popped by the other number. If the queue is not empty, both
    numbers act as they usually would.
* Though it did not make it to the competition, the problem writers also intended that in L3X mode,
  the input queue points downward.
* Many, if not all limitations in the original problem statement (such as the maximum number on a
  square) have been lifted, so that generally, as long as your computer can handle the computation,
  the IDE will compute it.