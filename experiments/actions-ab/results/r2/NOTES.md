# Round 2: notes for the write-up

- backstage-2 first started its backend in the tmux session `backstage-backend`, which backstage-1
  had created in its own sandbox. By backstage-2's account the command was typed into that pane
  but never ran there (backstage-1's backend was stopped and no log appeared), and it did not use
  the session again. backstage-1 had already finished. Shared tmux session names across sandboxes
  on one host are a harness isolation gap: a later round should give each run a session prefix.
