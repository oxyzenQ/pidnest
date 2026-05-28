# Pidnest

> Lightweight process inspection is now part of the Zejtron ecosystem.

## Migration Notice

`pidnest` has officially been merged into **Zejtron** and is no longer maintained as a standalone project.

All functionality now lives inside the `proc` module of Zejtron with improved performance, cleaner UX, and live monitoring support.

---

# Quick Start

## Inspect your own processes

```sh
zejtron proc --me
```

## Inspect another user

```sh
zejtron proc <USER_OR_UID>
```

## Live process monitoring

```sh
zejtron proc --me --live
```

---

# Why the move?

By integrating into Zejtron, process tooling now benefits from:

* Unified CLI ecosystem
* Faster updates and maintenance
* Shared infrastructure
* Better extensibility
* Live monitoring capabilities

---

# Install Zejtron

```sh
git clone https://github.com/oxyzenQ/zejtron
cd zejtron
```

---

# New Home

Repository:

https://github.com/oxyzenQ/zejtron

---

# Status

`pidnest` is archived and preserved for compatibility/reference purposes only.

All future development happens in **Zejtron**.
