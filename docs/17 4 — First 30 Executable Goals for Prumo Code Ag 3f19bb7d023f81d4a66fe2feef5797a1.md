# 17.4 — First 30 Executable Goals for Prumo / Code Agents

# Foundation

1. Bootstrap repository/toolchains.
2. Vendor/pin Prumo workforce.
3. Implement typed native IDs/value math.
4. Implement DocumentStore minimal hierarchy.
5. Implement Action/Command/Transaction/history.
6. Implement native error/result model and nanobind mapping.
7. Generate Python type stubs and strict typecheck.
8. Implement JobScheduler/cancellation skeleton.

# Persistence

1. Define PTND manifest/document schema v1.
2. Implement bounded package writer.
3. Implement loader/validator.
4. Implement atomic save/recovery test harness.

# UI

1. Create token compiler/theme foundation.
2. Implement MainWindow/tabs/action adapter.
3. Implement WorkspaceDockModel + Qt adapter.
4. Implement generic property schema/editor.
5. Implement Layers QAbstractItemModel.
6. Implement native CanvasHost frame scheduling.
7. Implement semantic UI inspection/accessibility baseline.

# Renderer/vector

1. Build renderer candidate benchmark harness.
2. Execute renderer spike and ADR.
3. Implement RenderScene rectangle/image primitives.
4. Implement Shape rectangle/ellipse creation.
5. Implement Move/Transform + hit testing.
6. Implement snapping baseline.
7. Implement fill/stroke/gradient baseline.
8. Implement SVG/PNG export.

# End-to-end

1. Implement first PTND vector golden workflow.
2. Implement Pen/Node path subsystem.
3. Run first integrated commercial-mini workflow gate.

# Per-goal contract

Every Goal references authority pages, agents/skills, criteria, tests, evidence and explicit non-goals.

[G001 — Bootstrap Repository & Toolchains](G001%20%E2%80%94%20Bootstrap%20Repository%20&%20Toolchains%203f19bb7d023f8182a1bbeb04a25611ef.md)

[G002 — Vendor & Pin Prumo Workforce](G002%20%E2%80%94%20Vendor%20&%20Pin%20Prumo%20Workforce%203f19bb7d023f81178c01ce1814070a27.md)

[G003 — Native Typed IDs, Math & Core Value Types](G003%20%E2%80%94%20Native%20Typed%20IDs,%20Math%20&%20Core%20Value%20Types%203f19bb7d023f8172bc55d634263c0089.md)

[G004 — Minimal Canonical DocumentStore & Hierarchy](G004%20%E2%80%94%20Minimal%20Canonical%20DocumentStore%20&%20Hierarchy%203f19bb7d023f816b925ce9474b767ddc.md)

[G005 — Actions, Commands, Transactions, History & ChangeSets](G005%20%E2%80%94%20Actions,%20Commands,%20Transactions,%20History%20&%20%203f19bb7d023f813c8c41d70b2aae7d20.md)

[G006 — Native Error Model & nanobind Exception Mapping](G006%20%E2%80%94%20Native%20Error%20Model%20&%20nanobind%20Exception%20Map%203f19bb7d023f814f8e19e24c08f26532.md)

[G007 — Python Type Stubs, Strict Typing & Native API Parity](G007%20%E2%80%94%20Python%20Type%20Stubs,%20Strict%20Typing%20&%20Native%20A%203f19bb7d023f819689f7e0b49ab37dba.md)

[G008 — JobScheduler & Cancellation Skeleton](G008%20%E2%80%94%20JobScheduler%20&%20Cancellation%20Skeleton%203f19bb7d023f81dbb611c6036933f3f6.md)

[G009 — PTND Manifest & Document Schema V1](G009%20%E2%80%94%20PTND%20Manifest%20&%20Document%20Schema%20V1%203f19bb7d023f812cae99d271c394b9d0.md)

[G010 — Bounded PTND Container Writer](G010%20%E2%80%94%20Bounded%20PTND%20Container%20Writer%203f19bb7d023f81fb9a67dac8e3260224.md)

[G011 — PTND Loader, Validator & Canonical Materialization](G011%20%E2%80%94%20PTND%20Loader,%20Validator%20&%20Canonical%20Material%203f19bb7d023f81bfad2ad926330a2137.md)

[G012 — Atomic Save, Fault Injection & Recovery Harness](G012%20%E2%80%94%20Atomic%20Save,%20Fault%20Injection%20&%20Recovery%20Har%203f19bb7d023f8157b73fcd0ed39b360c.md)

[G013 — Design Tokens, Theme Compiler & Component Foundation](G013%20%E2%80%94%20Design%20Tokens,%20Theme%20Compiler%20&%20Component%20F%203f19bb7d023f81709c9febb4d0dd10a7.md)

[G014 — MainWindow, Document Tabs & Action Adapter](G014%20%E2%80%94%20MainWindow,%20Document%20Tabs%20&%20Action%20Adapter%203f19bb7d023f814fa78fd12e3f2891e2.md)

[G015 — WorkspaceDockModel & Qt Docking Adapter](G015%20%E2%80%94%20WorkspaceDockModel%20&%20Qt%20Docking%20Adapter%203f19bb7d023f81ffb70cfada9decd2c3.md)

[G016 — Generic Property Schema & Inspector Framework](G016%20%E2%80%94%20Generic%20Property%20Schema%20&%20Inspector%20Framewo%203f19bb7d023f816691d5e3a3831c528f.md)

[G017 — Layers QAbstractItemModel & Semantic Drag/Drop](G017%20%E2%80%94%20Layers%20QAbstractItemModel%20&%20Semantic%20Drag%20D%203f19bb7d023f811a9327c6e050cb96a4.md)

[G018 — Native CanvasHost & Frame Scheduling](G018%20%E2%80%94%20Native%20CanvasHost%20&%20Frame%20Scheduling%203f19bb7d023f8165b7d9ef76e83153ed.md)

[G019 — Semantic UI Inspection & Accessibility Baseline](G019%20%E2%80%94%20Semantic%20UI%20Inspection%20&%20Accessibility%20Base%203f19bb7d023f8194be06feb1ac41fb7c.md)

[G020 — Renderer Benchmark Harness & Candidate Adapters](G020%20%E2%80%94%20Renderer%20Benchmark%20Harness%20&%20Candidate%20Adap%203f19bb7d023f8178a163f618a04e2596.md)

[G021 — Renderer Technology ADR & Production Backend Selection](G021%20%E2%80%94%20Renderer%20Technology%20ADR%20&%20Production%20Backen%203f19bb7d023f8103bae1d2653b4552ce.md)

[G022 — Production RenderScene & Primitive Renderer](G022%20%E2%80%94%20Production%20RenderScene%20&%20Primitive%20Renderer%203f19bb7d023f81a1b6d8e99f5a38482d.md)

[G023 — Parametric Rectangle/Ellipse Creation & Rendering](G023%20%E2%80%94%20Parametric%20Rectangle%20Ellipse%20Creation%20&%20Ren%203f19bb7d023f81e68713d5fa0d18e322.md)

[G024 — Move / Transform, Selection & Hit Testing](G024%20%E2%80%94%20Move%20Transform,%20Selection%20&%20Hit%20Testing%203f19bb7d023f81bb825be40541162321.md)

[G025 — Snapping, Guides & Precision Baseline](G025%20%E2%80%94%20Snapping,%20Guides%20&%20Precision%20Baseline%203f19bb7d023f8134b701ed36f6911c5f.md)

[G026 — Fill, Stroke & Gradient Appearance Baseline](G026%20%E2%80%94%20Fill,%20Stroke%20&%20Gradient%20Appearance%20Baseline%203f19bb7d023f81189633e69521841a3b.md)

[G027 — SVG & PNG Export Baseline](G027%20%E2%80%94%20SVG%20&%20PNG%20Export%20Baseline%203f19bb7d023f81b2bb7cc18246823706.md)

[G028 — First PTND Vector Golden Workflow](G028%20%E2%80%94%20First%20PTND%20Vector%20Golden%20Workflow%203f19bb7d023f81ac9802cf800505a024.md)

[G029 — Pen, Node & Vector Path Geometry Subsystem](G029%20%E2%80%94%20Pen,%20Node%20&%20Vector%20Path%20Geometry%20Subsystem%203f19bb7d023f81fda8eddff0d36a4f7e.md)

[G030 — First Integrated Commercial-Mini Workflow Gate](G030%20%E2%80%94%20First%20Integrated%20Commercial-Mini%20Workflow%20G%203f19bb7d023f814c9d9adbf13cf53fcc.md)

[G01 — Bootstrap Repository & Toolchains](G01%20%E2%80%94%20Bootstrap%20Repository%20&%20Toolchains%203f19bb7d023f81bdab71c94259e21e14.md)

[G02 — Vendor & Pin Prumo Workforce](G02%20%E2%80%94%20Vendor%20&%20Pin%20Prumo%20Workforce%203f19bb7d023f81efbb17cca1b2dad54f.md)

[G03 — Typed IDs, Math & Core Value Types](G03%20%E2%80%94%20Typed%20IDs,%20Math%20&%20Core%20Value%20Types%203f19bb7d023f81da9e2ff2fc2704ee4f.md)

[G04 — Minimal Canonical DocumentStore](G04%20%E2%80%94%20Minimal%20Canonical%20DocumentStore%203f19bb7d023f810e82fdd103db7256ae.md)

[G05 — Actions, Commands, Transactions & History Kernel](G05%20%E2%80%94%20Actions,%20Commands,%20Transactions%20&%20History%20Ke%203f19bb7d023f81488047f0207d6d5d9b.md)

[G06 — Native Error Model & nanobind Boundary](G06%20%E2%80%94%20Native%20Error%20Model%20&%20nanobind%20Boundary%203f19bb7d023f8190a10beff4c54f1695.md)

[G07 — Python Type Stubs & Strict Application Contracts](G07%20%E2%80%94%20Python%20Type%20Stubs%20&%20Strict%20Application%20Contr%203f19bb7d023f8153a293d9a34ff61e6a.md)

[G08 — JobScheduler & Cancellation Skeleton](G08%20%E2%80%94%20JobScheduler%20&%20Cancellation%20Skeleton%203f19bb7d023f81cfb41ff85213455769.md)

[G09 — PTND Manifest & document.json Schema v1](G09%20%E2%80%94%20PTND%20Manifest%20&%20document%20json%20Schema%20v1%203f19bb7d023f8184b103c7b1400ae1d3.md)

[G10 — Bounded PTND Package Writer](G10%20%E2%80%94%20Bounded%20PTND%20Package%20Writer%203f19bb7d023f816faafcc336a1ed31e5.md)

[G11 — PTND Loader, Validator & Capability Negotiation](G11%20%E2%80%94%20PTND%20Loader,%20Validator%20&%20Capability%20Negotiat%203f19bb7d023f814f880cd371c2d6576c.md)

[G12 — Atomic Save, Recovery Journal & Fault-Injection Harness](G12%20%E2%80%94%20Atomic%20Save,%20Recovery%20Journal%20&%20Fault-Inject%203f19bb7d023f81e2881fcd0f5fb8b061.md)

[G13 — Design Tokens, Theme Compiler & Component Foundation](G13%20%E2%80%94%20Design%20Tokens,%20Theme%20Compiler%20&%20Component%20Fo%203f19bb7d023f8101b29df14b54a71274.md)

[G14 — MainWindow, Tabs & QAction Adapter](G14%20%E2%80%94%20MainWindow,%20Tabs%20&%20QAction%20Adapter%203f19bb7d023f815fad82cbcc8d05a1e6.md)

[G15 — WorkspaceDockModel & Qt Dock Adapter](G15%20%E2%80%94%20WorkspaceDockModel%20&%20Qt%20Dock%20Adapter%203f19bb7d023f81b3a157e30884d161fb.md)

[G16 — PropertySchema Registry & Generic Inspector](G16%20%E2%80%94%20PropertySchema%20Registry%20&%20Generic%20Inspector%203f19bb7d023f8126ab2bdbc45189978b.md)

[G17 — Layers QAbstractItemModel & Selection Synchronization](G17%20%E2%80%94%20Layers%20QAbstractItemModel%20&%20Selection%20Synchr%203f19bb7d023f8172858fcead4fa3bc08.md)

[G18 — Native CanvasHost & Frame Scheduling](G18%20%E2%80%94%20Native%20CanvasHost%20&%20Frame%20Scheduling%203f19bb7d023f815e9ae6f07ac157d740.md)

[G19 — Semantic UI Inspection & Accessibility Baseline](G19%20%E2%80%94%20Semantic%20UI%20Inspection%20&%20Accessibility%20Basel%203f19bb7d023f812cb3edfb93bc5db50f.md)

[G20 — Renderer Candidate Benchmark Harness](G20%20%E2%80%94%20Renderer%20Candidate%20Benchmark%20Harness%203f19bb7d023f8179807cc987f34a620d.md)

[G21 — Renderer Decision ADR & Backend Selection](G21%20%E2%80%94%20Renderer%20Decision%20ADR%20&%20Backend%20Selection%203f19bb7d023f8111bfedc3bf5582c75b.md)

[G22 — Minimal RenderScene & Rectangle/Image/Text Placeholder Primitives](G22%20%E2%80%94%20Minimal%20RenderScene%20&%20Rectangle%20Image%20Text%20P%203f19bb7d023f8182b91ee7e03dc84bd2.md)

[G23 — Rectangle/Ellipse Parametric Shape Creation](G23%20%E2%80%94%20Rectangle%20Ellipse%20Parametric%20Shape%20Creation%203f19bb7d023f8138896dddb5273e3301.md)

[G24 — Move/Transform & Hit Testing](G24%20%E2%80%94%20Move%20Transform%20&%20Hit%20Testing%203f19bb7d023f8147bf25fd4021330e1e.md)

[G25 — Snapping & Smart Guides Baseline](G25%20%E2%80%94%20Snapping%20&%20Smart%20Guides%20Baseline%203f19bb7d023f815fa6d0e37f7da5e264.md)

[G26 — Fill, Stroke & Gradient Baseline](G26%20%E2%80%94%20Fill,%20Stroke%20&%20Gradient%20Baseline%203f19bb7d023f81e980edf3b169483690.md)

[G27 — SVG & PNG Export Baseline](G27%20%E2%80%94%20SVG%20&%20PNG%20Export%20Baseline%203f19bb7d023f810fa98cecc5969dd45b.md)

[G28 — First PTND Vector Golden Workflow](G28%20%E2%80%94%20First%20PTND%20Vector%20Golden%20Workflow%203f19bb7d023f81deab5bffafd7f63f25.md)

[G29 — Pen/Node Path Subsystem](G29%20%E2%80%94%20Pen%20Node%20Path%20Subsystem%203f19bb7d023f814f87ebeb5a33ad9670.md)

[G30 — First Integrated Commercial-Mini Gate](G30%20%E2%80%94%20First%20Integrated%20Commercial-Mini%20Gate%203f19bb7d023f8120b786ddb59fc7bcc4.md)