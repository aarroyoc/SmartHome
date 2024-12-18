# SmartHome
Collection of Smart Home programs and utilities


## Sample uLisp program

```
(defun blink (x)
  (pinmode 2 t)
  (digitalwrite 2 x)
  (delay 1000)
  (blink (not x)))

(blink t)
```