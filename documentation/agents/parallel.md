# Parallel sessions

When more than one Claude session works on this repository at once, each works in a folder of its own, and one session, the **integrator**, makes every commit and every push. A shared folder means each session's checks also run over the other's half-done work, and two sessions end up fixing the same failure twice.

## Folders and branches

- The integrator works in the main checkout, `~/Documents/engineering/board`, on `main`.
- Every other session gets a git worktree at `~/Documents/engineering/worktrees/<area>`, on a branch named after its area, created from `main`:

  ```sh
  git worktree add -b <area> ~/Documents/engineering/worktrees/<area> main
  bun install --cwd ~/Documents/engineering/worktrees/<area>
  ```

- A session edits files only inside its own folder. Its own `bun run check` then sees only its own work.
- Each session owns an area of files, agreed by message when it starts. A change outside that area is agreed with the integrator first.

## Handing work over

1. The session finishes a piece of work, runs `bun run check` in its own folder, and messages the integrator the files and the commit message it suggests.
2. The integrator commits those files on the session's branch, in the session's folder: `git -C <folder> commit`.
3. The integrator applies the commit to `main` in the main checkout, pushes, and messages back the hash.
4. The integrator rebases the session's branch onto `main` (`git -C <folder> rebase --autostash main`), so the session works on top of everything that has landed.

A hand-over is done when its commit is on `main` on GitHub and the session has the hash.

## Ending a session

Once its branch has nothing left that is not on `main`, the integrator removes its folder and its branch:

```sh
git worktree remove ~/Documents/engineering/worktrees/<area>
git branch -d <area>
```
