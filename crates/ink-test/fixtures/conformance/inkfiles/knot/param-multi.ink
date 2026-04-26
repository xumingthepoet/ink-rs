VAR x: int = 1
        VAR y: string = "Hmm."
        How much do you give?
        * [I don't know] -> give(x, 2, y)
        
        === give(a, b, c) ===
            You give {a} or {b} dollars. {y}
            -> END