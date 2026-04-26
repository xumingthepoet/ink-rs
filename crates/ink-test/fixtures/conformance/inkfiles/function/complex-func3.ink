
VAR weight: int = 20
VAR roll: int = 0
VAR mult: int = 1
VAR dst: int = 5
VAR deadline: int = 0
VAR fee: int = 0

     ~ merchant_init()
      "I will pay you {fee} reales if you get the goods to their destination. The goods will take up {weight} cargo spaces."
     -> END
     
     === function merchant_init() => void
     
     { roll == 0:
        ~ mult = 2
     }
     
     { mult == 2:
        ~ roll = 1
     }
     
     { roll == 0:
        ~ mult = 3
     }
     
     ~ deadline = (dst * (100)) / 100
     ~ fee = (1 + dst) * 10 * mult
