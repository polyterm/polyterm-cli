Output demo.gif

Set FontSize 14
Set Width 1280
Set Height 720
Set Theme "Catppuccin Mocha"
Set TypingSpeed 50ms

Sleep 500ms
Type "./target/release/polyterm"
Sleep 300ms
Enter
Sleep 2s

# In menu view, Markets is already highlighted — open it
Enter
Sleep 4s

# Scroll through a few markets
Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 600ms

# Open the live order book
Enter
Sleep 5s

# Flip to the NO outcome
Tab
Sleep 3s

# Back to markets
Escape
Sleep 800ms

# Back to menu
Escape
Sleep 500ms

# Quit
Type "q"
Sleep 800ms
