# Chapter 1 — What is Spanda, anyway?

Back to [index](./README.md)

---

## The problem (without the buzzwords)

Building a robot today usually means:

1. A Python script that calls an LLM
2. A C++ node that talks to the motor driver
3. A ROS topic somewhere in the middle
4. A safety person asking “who stops the robot if the model hallucinates?”
5. A deploy checklist nobody updates

Nothing is *wrong* with that stack — it's just **a lot of duct tape**.

Spanda is an **autonomous systems platform** — and at its center is one language (`.sd` files) where sensors, AI, safety rules, and “will this fit on the Jetson?” live in the **same program**. The same toolchain also verifies hardware, simulates missions, and monitors health.

---

## The body metaphor

Think of a robot like a body:

| Part | In real life | In Spanda |
|------|--------------|-----------|
| Senses | Eyes, lidar, IMU | `sensor` |
| Muscles | Wheels, arm, gripper | `actuator` |
| Reflexes | “Don't hit the wall” | `safety { }` |
| Brain | Planner, LLM, vision | `ai_model`, `agent` |
| Bouncer | Only safe moves get through | `safety.validate()` |
| Nervous system | The program that ties it together | `behavior`, `task` |

Spanda is the **pulse** — the thing that turns perception into action safely.

---

## What a `.sd` file is

- **`.sd`** = Spanda source file (think “system definition”)
- One file can describe one robot, or a whole fleet
- You **check** it (compiler), **run** it (simulator), **verify** it (hardware fit)

You are not writing a web app. You are writing **what the robot is allowed to do**.

---

## What Spanda is NOT

| Myth | Reality |
|------|---------|
| “It's Python with robots” | It's its own language with robot keywords built in |
| “It replaces ROS” | It can talk ROS-style topics; it's the program layer on top |
| “The AI drives the robot” | AI *proposes*; safety *approves*; actuators only see approved moves |
| “I need a real robot to try it” | `spanda run` uses a simulated backend |

---

## The safety rule everyone remembers

```spanda
// ❌ Compiler says no
wheels.execute(proposal);

// ✅ This is the pattern
let action = safety.validate(proposal);
wheels.execute(action);
```

If you remember one thing from this guide: **AI output is untrusted until validated.**

---

## Try it now

No setup essay — just run an example from the repo:

```bash
spanda check examples/basics/01_minimal_robot.sd
spanda run examples/basics/02_sensors_and_safety.sd
```

If those work, you understand 80% of what Spanda is for.

---

**Next:** [Your first five minutes](./02-five-minutes.md)
