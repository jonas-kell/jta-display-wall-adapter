# Display Wall Module by JTA

Software used for running custom display walls.

## Dev

```cmd
docker compose up
```

## Cross compile Windows executable on Linux

```cmd
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
```

## Start on a Windows live system

-   Copy over the executable from the previous compilation step (`/target/x86_64-pc-windows-gnu/release/jta-display-wall-adapter.exe`)
-   Also copy the batch file `/start-jta-display-wall-adapter.bat` and the icon `/Link-Icon.ico`
-   Put that together in a folder somewhere you desire
-   Now right click on the .bat file "Send to Desktop (create a shortcut)"
-   Edit the shortcut and prepend/edit the location with `C:\Windows\System32\cmd.exe /c "<...this is the content that was previously in the field...>"`
    -   Now this is seen as an executable and you can pin it to the taskbar by dragging
    -   Now for this shortcut you can set the icon
