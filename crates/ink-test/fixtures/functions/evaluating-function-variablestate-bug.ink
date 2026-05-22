=== module game ===

== main ==
Start
 -> tunnel ->
 End


 == tunnel ==
 In tunnel.
 ->->

 == function function_to_evaluate() => string ==
     { if zero_equals_(1):
         ~ return "WRONG"
     - else:
         ~ return "RIGHT"
     }

 == function zero_equals_(k: int) => bool ==
     ~ do_nothing(0)
     ~ return  (0 == k)

 == function do_nothing(k: int) => int ==
     ~ return 0
