# Persistent UltraGraph store, stage 1: the embedded store

* Date: 2026-10-16
* Status: draft specification
* Parent: `openspec/notes/ultragraph/PersistentStore.md` (§10 for the stages, §13 for identifiers)
* Scope: stage 1 only. One process holds one store in one directory with one replica. The scope
  covers the engine, its on-disk format, and the simulator that tests it. Stage 2 (the single-node
  server) reuses the engine and the simulator.
* Licence of the implementation: MIT. The implementation is written from this specification under
  the clean-room rules of §1.2, and §2 lists every outside source of an idea it uses.

## 1. Specification, tests and implementation link to each other

### 1.1 The chain

Every requirement below carries an identifier, for example `S1-LOG-4`, and two lines:

- **Source** names where the idea comes from. A source outside this project is listed in §2.
  `this project` marks UltraGraph or DeepCausality, and `this design` marks a decision made here.
  `common technique` marks a practice with no single source. `DeepBrain` marks a requirement of
  DeepBrain, the store's first consumer, taken from its design outside this repository. It is a
  requirement, not a borrowed idea, so §2 does not list it.
- **Verified by** states what the tests must show.

Each layer links to the one before it:

1. **The specification** gives every requirement an identifier and a source.
2. **The tests** name the requirements they verify. Their names start with the identifier in snake
   case (`s1_log_4_queued_batches_share_one_barrier`), and their doc comments repeat it
   (`/// Verifies: S1-LOG-4`).
3. **The implementation** names, in each module's and each public item's doc comment, the
   requirements it implements and the test files that verify them
   (`/// Implements: S1-LOG-4. Verified by: tests/log/group_commit_tests.rs`).

Every implementation item therefore leads to a test, every test to a requirement, and every
requirement to its source.

- A requirement without a test is unfinished.
- A test without a requirement either exposes a missing requirement or is redundant.
- A script lists both cases (proposed).

The tests come before the implementation, in the five phases of DeepCausality's TDD protocol
(`openspec/specs/unified-math-tdd-protocol/spec.md`; §20).

### 1.2 Clean-room rules

- The implementation is written from this specification. It copies no source code from the projects
  in §2, and their source files stay closed while the implementation is written.
- This specification restates each idea it takes in its own words. It quotes a source only where the
  quotation is the attribution.
- If a file ever starts from copied code:
  - it keeps its original licence header and carries a notice that it was modified;
  - the repository's `NOTICE` file gains the original project's NOTICE text, as Apache-2.0 §4
    requires;
  - that file is then Apache-2.0, and the Source line of each requirement it implements says so.

## 2. Attribution

### 2.1 Rules

Attribution here is minimal and true:
- **Name a source only for an idea this design takes from it,** and state the idea.
- **Name the exact material and the way it was read:** source at a commit, documentation pages,
  a paper read in full, or a summary.
- **Make no comparative claim about a named project.** Not "compatible with", "equivalent to",
  "faster than" or "based on" beyond the idea stated.
- **Mark an idea that has no specific source** as `this design` or `common technique`.
- **Keep the README's "Prior art" section (§2.3) worded as §2.2 is.**

### 2.2 Sources

| Source | Material read (2026-10-16) | Licence | Ideas taken | Requirements |
|---|---|---|---|---|
| Apache Iggy | Source at commit `fe3b4269` (2026-10-15): `core/journal`, `core/partitions`, `core/metadata`, `core/simulator` | Apache-2.0 | Data owned by one shard and reached by messages, with consensus on single-threaded shards; the shape of the storage interface, with production code written against it and simulated behind it; the two-copy superblock and its replacement procedure; refusing a superblock copy that fails verification; the rule for repairing a damaged log end; group-commit bounds; log segment file names; time in apply only from the record; the simulated filesystem model; the seeded deterministic executor; fault sweeps over every storage operation, asserting that acknowledged history survives; removing obsolete files after restart; a shadow model with an oracle | S1-ARCH-2, S1-ARCH-4, S1-STOR-1, S1-SB-1–3, S1-REC-3, S1-LOG-2–5, S1-APP-2, S1-RECOV-2, S1-RECOV-3, S1-SIM-1–4, S1-SIM-6–9 |
| TigerBeetle | Documentation: `docs/internals/vsr.md`, `docs/internals/data_file.md`, and the pages Performance, Safety and Upgrading on docs.tigerbeetle.com | Apache-2.0 | One thread applying batches; making a batch durable before executing it; checksummed, hash-chained records; fixed-size records; batching; checkpointing by switching a root record; deterministic compaction; timestamps assigned at sequencing; the storage failure modes latent sector error, silent corruption and misdirected I/O | S1-ARCH-1, S1-APP-1, S1-APP-2, S1-REC-1, S1-REC-2, S1-LOG-4, S1-CKPT-1, S1-CKPT-6, S1-SIM-5 |
| LMAX | Martin Fowler, *The LMAX Architecture* (martinfowler.com) | Article | A ring buffer whose consumers run in parallel and gate each other by watching counters | S1-ARCH-5 |
| GraphOne (Kumar and Huang, ACM TOS 15(4), 2019) | Paper read in full | Paper | A read view made of a snapshot plus the log after it | S1-READ-2 |
| TAO (Bronson et al., USENIX ATC 2013) | Paper, §4.1 read | Paper | A placement identifier embedded in each object identifier | S1-ID-1, S1-ID-2 |
| ULID | Specification at github.com/ulid/spec | Specification | A 48-bit millisecond timestamp as the identifier's leading field | S1-ID-2 |
| LSMGraph (PACMMOD 2024), BACH (PVLDB 2025) | Summaries in `PersistentStore.md` §3.4; the papers were not read in full | Papers | An in-memory adjacency level over immutable CSR runs, merged by compaction | S1-CKPT-3 |

### 2.3 README text

> **Prior art.** This store takes ideas, not code, from these sources:
> - **Apache Iggy:** data owned by one shard and reached by messages, the shape of the storage
>   interface, the two-copy superblock procedure, the rule
>   for repairing a damaged log end, group-commit bounds, segment file names, and the design of its
>   simulator: the filesystem model, the seeded executor, fault sweeps, and the shadow model with an
>   oracle.
> - **TigerBeetle:** one thread applying batches, making a batch durable before executing it,
>   checksummed and hash-chained records, fixed-size records, batching, checkpointing by switching a
>   root record, deterministic compaction, and its list of storage failure modes.
> - **LMAX:** a ring buffer whose consumers gate each other by watching counters.
> - **GraphOne:** a read view made of a snapshot plus the log after it.
> - **TAO:** placement embedded in identifiers.
> - **ULID:** a millisecond timestamp leading the identifier.
> - **LSMGraph and BACH:** an in-memory adjacency level over immutable CSR runs.
>
> No source code from these projects is included.

The README carries the last sentence only while it is true (§1.2).

## 3. Scope

**In stage 1:**
- one engine in one process, owning one store directory;
- the whole graph in memory in UltraGraph's Live state;
- the log, checkpoints, recovery and snapshot reads;
- the storage interface with two implementations, on std and on the simulator;
- the requirements DeepBrain sets for transactions, indexes, queries, the change feed, declarations,
  namespaces, retraction and personal data (§13 to §17);
- the simulator, DeepBrain's conformance kit, and the measurements of §19.

**Later stages:**
- the network server (stage 2);
- replication (stage 3);
- object storage and graphs larger than memory (stage 4);
- the write grid (stage 5);
- distributed analytics;
- access control, which waits for DeepBrain's security architecture.

## 4. The engine is one thread behind a storage interface

**S1-ARCH-1 One owner.** One engine owns the graph and the log and applies batches on one thread.
- Source: TigerBeetle (Performance: "a single core by design").
- Verified by: the determinism tests of S1-APP-2.

**S1-ARCH-2 Storage only through the interface.** The engine reaches files and directories only
through the storage interface (S1-STOR-1), so the same engine runs on the std implementation and on
the simulator.
- Source: Apache Iggy (its journal code is written against `DurableStorage`, which its simulator
  implements).
- Verified by: every simulator test runs the production engine.

**S1-ARCH-3 A thread-safe facade.** The engine core uses single-threaded types. Callers reach it
through a facade that hands requests to the engine thread and returns futures that are `Send`, as
`ContextStorage` requires.
- Source: this design.
- Verified by: facade tests from several threads.

**S1-ARCH-4 One shard, no imposed runtime, work overlapped with the barrier.**
- **One shard.** The engine is one shard of the stage-2 architecture. It owns the graph and the log,
  and it is reached only through messages: requests in, replies out. Stage 2 places this shard on a
  core and adds connection shards; stage 5 runs one write group per shard.
- **No imposed runtime.** Stage 1 is a library inside a host application. It creates at most two
  threads, the engine thread and an optional background compaction thread, through a spawner the
  host can supply. It sets no core affinity and requires no particular async runtime. Pinning cores
  and choosing io_uring belong to the stage-2 server.
- **Overlapping the barrier.** While the barrier for group k is in flight, the engine applies group
  k − 1 and sequences and encodes group k + 1. The write and `sync_data` of each group run on one I/O
  task.
  - A group is applied only after its own barrier succeeds (S1-APP-1).
  - Acknowledgements leave in log order.
  - Under the simulator, the I/O task is one more task on the seeded executor (S1-SIM-6).
- Source: Apache Iggy (each partition owned by one shard; consensus on single-threaded shards);
  this design (a library that imposes no runtime; overlapping work with the barrier).
- Verified by:
  - every engine operation reachable only through the message interface;
  - a counting spawner observes at most two threads;
  - with the file sync paused in the simulator, the engine sequences and encodes the next group but
    neither applies nor acknowledges the group whose barrier is pending;
  - after the barrier resumes, acknowledgements arrive in log order;
  - S1-MEAS-2 measures commit throughput and latency with the overlap on and off.

**S1-ARCH-5 A bounded ring of batch slots coordinates the pipeline.**
- **Slots and cursors.** Sequencing, the barrier and apply share one ring of preallocated batch
  slots. Three cursors advance over it in order:
  - sequenced, advanced by the engine;
  - durable, advanced by the I/O task after a barrier succeeds;
  - applied, advanced by the engine.

  Acknowledgements follow the applied cursor, so they leave in log order by construction.
- **Groups.** The I/O task takes every slot between the durable and the sequenced cursor as one group,
  so group commit (S1-LOG-4) falls out of the ring.
- **Backpressure.** The ring's capacity bounds the memory of batches in flight. A full ring is where
  callers wait.
- **Waiting.** Waiting uses wakers, never a spinning loop, so an embedded host loses no CPU, and the
  ring runs as tasks on the simulator's seeded executor (S1-SIM-6).
- **Ownership.** No slot is shared between threads while it is in use: a slot's buffer moves between
  the engine and the I/O task with ownership of it, for example as fixed buffers circulating through
  two bounded channels. That needs no `unsafe` code.
- Source: LMAX (Fowler's article: a ring buffer whose consumers run in parallel and gate each other
  by watching counters); this design (waking instead of spinning; buffers moved by ownership).
- Verified by:
  - the cursors never cross: applied ≤ durable ≤ sequenced, under random seeds;
  - a full ring blocks callers without loss, and they resume when slots free;
  - no allocation per batch after warm-up, by counting buffer creations;
  - a group never spans a failed barrier: on failure, no slot between the durable and the sequenced
    cursor is acknowledged (S1-LOG-5).

## 5. Identifiers carry their slot, which is zero in stage 1

**S1-ID-1 Vertex identifiers.** A vertex identifier is a `u64`: b slot bits, then a local index of
64 − b bits.
- In stage 1 the slot is 0.
- The local index is the vertex's UltraGraph node position, never reissued.
- b is proposed as 10 (§21).
- Source: TAO (§4.1, placement embedded in the identifier); this project (permanent node positions).
- Verified by: encode/decode round trips at the field boundaries; a removed vertex's position is
  never issued again, across restarts.

**S1-ID-2 Edge identifiers.** An edge identifier is a `u128`, in this order:
1. milliseconds since the Unix epoch, 48 bits;
2. the source vertex's slot, b bits;
3. the target vertex's slot, b bits;
4. a random suffix, 80 − 2b bits.

- Source: ULID (the 48-bit millisecond timestamp in front); TAO (placement in the identifier).
- Verified by: field round trips; identifiers sort by their time field.

**S1-ID-3 Reservations.** A client reserves vertex local indices in blocks. The engine logs each
reservation (S1-REC-5), so a reserved index is never issued twice, across restarts.
- Source: this design; this project (the shape of `ContextStorage::reserve`).
- Verified by: reserve, crash, recover, reserve again; the two ranges are disjoint.

**S1-ID-4 An insert is new.** Inserting an edge whose identifier already exists is refused. It is
never treated as an update; updating an edge is its own operation (S1-REC-5). In stage 1 the whole
graph is in memory, so the check is exact.
- Source: this design.
- Verified by: a second insert with the same identifier is refused, and the first edge is unchanged.

**S1-ID-5 Natural keys.** A key the store does not choose (a git blob SHA, a DOI, a URL, an external
identity) selects its slot by hashing, and a unique index inside the slot (S1-IDX-2) leads from the
key to the local index. In stage 1, with one slot, that is one unique index.
- Source: DeepBrain (identities created by upsert); this design.
- Verified by: an upsert by natural key creates once and then finds the same node, across restarts.

## 6. Log records are fixed-size, checksummed and hash-chained

**S1-REC-1 Batch records.** The log is a sequence of batch records. Each header holds:
- the format version;
- the first operation number and the operation count;
- the batch timestamp (S1-APP-2);
- the body length;
- a checksum of the body;
- the checksum of the previous record's header;
- a checksum of the header itself.

- Source: TigerBeetle (Safety: data is "checksummed, and hash-chained"; `vsr.md`, hash-chained
  headers).
- Verified by: a corrupt header, a corrupt body, a missing record and a reordered record are each
  detected.

**S1-REC-2 Body layout.** A body is an array of fixed-size mutation records (proposed 48 bytes: an
operation code, flags, the source, the target, an edge identifier or key, and a value reference),
followed by a value area that holds the encoded bytes of new payloads.
- Source: TigerBeetle (fixed-size records, Performance); keeping values beside the records is a
  common technique.
- Verified by: round trips; every mutation record has the same size.

**S1-REC-3 Repairing the log end.** On recovery, damage at the end of the log is truncated only when
no complete, valid record follows it. If a complete, valid record follows the damage, the store
refuses to open and reports the position, because that record was durable and may have been
acknowledged.
- Source: Apache Iggy (`core/journal/src/prepare_journal.rs`, `truncate_or_fail`).
- Verified by: a silent torn write of the last record is truncated; damage in the middle with a valid
  record behind it is refused.

**S1-REC-4 Payload encoding.** The node payload `N` and the edge payload `W` are encoded by a codec
trait, and a value always encodes to the same bytes.
- Source: this project (the `Storable` trait in `deep_causality_context`).
- Verified by: round trips for every payload kind, including floating-point bit patterns.

**S1-REC-5 Operations.** The mutation set is: create node, update node, retract node, insert edge,
update edge, retract edge, reserve identifiers, and checkpoint (S1-CKPT-2).
- Source: this project (UltraGraph's `GraphMut` and `ContextStorage` operations); this design.
- Verified by: one encode/decode/apply test per operation.

## 7. The log is appended, and acknowledged only after a shared barrier

**S1-LOG-1 Append-only segments.** Log segments are files under `wal/`. A segment is only ever
appended to, and it never changes once a later segment exists.
- Source: common technique.
- Verified by: the trace of storage operations shows no write behind a segment's end and no write to
  a closed segment.

**S1-LOG-2 Segment names.** A segment's file name is its first operation number in 20 decimal
digits, zero-padded, with the extension `.wal`.
- Source: Apache Iggy (`{start_offset:020}.log`, `core/partitions/src/io.rs`).
- Verified by: name formatting and parsing at 0 and at `u64::MAX`.

**S1-LOG-3 A new segment is durable only after its directory sync.** After creating a segment, the
engine syncs `wal/` before acknowledging any batch in that segment.
- Source: Apache Iggy (the simulator test asserting "file sync must not imply directory sync").
- Verified by: a power loss after the file sync but before the directory sync loses the segment, and
  no batch in it was acknowledged.

**S1-LOG-4 Group commit.** Batches queued while a barrier is in flight share the next barrier: one
write and one `sync_data` cover the group. A group is bounded by a batch count and a byte budget,
starting at 256 batches and 8 MiB.
- Source: Apache Iggy (the group-commit bounds in `core/partitions/src/persistence.rs`); TigerBeetle
  (Performance: "the cost of replication is paid only once per batch").
- Verified by: N queued batches produce one file sync in the trace; both bounds hold at, below and
  above their limits.

**S1-LOG-5 Acknowledge after the barrier, never after a failed one.**
- A batch is acknowledged only after the barrier covering it succeeds.
- When a barrier fails, no batch of its group is acknowledged, and the engine stops accepting writes
  until the store is reopened and recovered. A failed sync is not retried.
- Source: Apache Iggy (the simulator's writeback-failure model and the test
  `failed_group_barrier_never_acknowledges_a_partial_batch`).
- Verified by: a fault at the file sync, and a writeback failure reported by a later sync, each leave
  every batch of the group unacknowledged.

## 8. Apply runs after durability and is deterministic

**S1-APP-1 Durable, then applied.** A batch is applied to the graph only after it is durable. A
reader never observes a mutation that is not in the durable log.
- Source: TigerBeetle (`vsr.md`: a prepare is durable before it commits).
- Verified by: a crash between the barrier and apply recovers the batch; no reader observes state
  beyond the durable log.

**S1-APP-2 Deterministic apply.** Apply is a function of the state and the batch alone. It reads no
clock other than the batch timestamp, draws no random number, iterates no unordered collection, and
never depends on an allocation address.
- The engine assigns the batch timestamp at sequencing, never below its predecessor's.
- Source: Apache Iggy (`core/metadata/src/stm/mod.rs`: apply must be deterministic, and the primary's
  timestamp is its only clock); TigerBeetle (`vsr.md`: the primary assigns timestamps).
- Verified by: the same log replayed into two fresh engines yields equal state digests (S1-APP-5),
  for random logs from a seeded generator.

**S1-APP-3 Deterministic refusals.** A refused mutation is refused identically on replay, and the
batch result records each refusal. Refusals include an edge to a retracted node and an insert with an
existing identifier.
- Source: this design.
- Verified by: logs containing refused mutations replay to the same results and digests.

**S1-APP-4 Retraction without fan-out.** Retracting a node clears its liveness bit. Its edges stay in
place until compaction, and readers skip edges whose endpoint is retracted.
- Source: this project (the Live state's mask bit per node).
- Verified by: after a retraction no read returns the node's edges; after compaction the edges are
  gone from the segments.

**S1-APP-5 State digest.** The engine computes a digest of the graph that depends only on its content
(nodes, edges and payloads, in identifier order) and not on its memory layout.
- Source: this design.
- Verified by: equal graphs built in different orders have equal digests; graphs differing in one
  edge or payload byte do not.

## 9. The superblock has two copies, replaced by rename

**S1-SB-1 Contents.** Two files, `superblock.0` and `superblock.1`. Each holds:
- the format version and a sequence number;
- the checkpoint's operation number and manifest hash;
- the position and header checksum of the checkpoint's log record;
- the checksum of the parent superblock record;
- its own checksum.

- Source: Apache Iggy (`core/journal/src/superblock.rs`, `PingPongSuperblock`).
- Verified by: round trips; a flipped bit in any field fails verification.

**S1-SB-2 Replacement.** To update the superblock:
1. write the new record to a temporary file;
2. sync it;
3. rename it over the older copy;
4. sync the directory.

- Source: Apache Iggy (same file).
- Verified by: a fault at each of the four steps, under both crash kinds, leaves at least one valid
  copy holding the old or the new record.

**S1-SB-3 Choosing a copy.** Opening the store takes the valid copy with the higher sequence. A copy
that fails verification blocks writes until resolved, because its unreadable sequence may belong to
the newer generation.
- Source: Apache Iggy (same file).
- Verified by: one corrupt copy blocks writes; two valid copies yield the higher sequence.

## 10. A checkpoint writes immutable segments and then switches the superblock

**S1-CKPT-1 What a checkpoint is.** A checkpoint records the graph as of operation N in immutable
segment files, and switches the store to them by updating the superblock.
- Source: TigerBeetle (`data_file.md`: a checkpoint switches the superblock's root).
- Verified by: S1-CKPT-2 and S1-RECOV-1.

**S1-CKPT-2 Order.**
1. Write the segment files, each synced: forward CSR runs sorted by (source, target, key), backward
   runs sorted by (target, source, key).
2. Sync `segments/` (and `values/` if new value files exist).
3. Write the manifest, sync it, and sync `manifests/`.
4. Append a checkpoint record `{ N, manifest hash }` to the log through group commit.
5. Update the superblock (S1-SB-2).
6. Delete the log segments wholly before the checkpoint record, and every segment file that the
   current manifest does not reference.

- Source: this design. That the barriers complete before the log is reclaimed is also what Iggy's
  test `checkpoint_barriers_complete_before_wal_reclamation` checks for its own checkpoint.
- Verified by: a fault sweep (S1-SIM-7) over a checkpoint recovers either the old checkpoint plus its
  log or the new checkpoint plus its log, never a mixture.

**S1-CKPT-3 Incremental checkpoints.** A checkpoint writes as a delta segment only the rows changed
since the previous one. Compaction merges delta segments into a base segment, and the manifest lists
the base and its deltas in order.
- Source: LSMGraph and BACH (summaries; §2.2).
- Verified by: a graph recovered from a base plus deltas equals the graph recovered from one full
  segment (by digest, S1-APP-5).

**S1-CKPT-4 Content-addressed names.** File names are content hashes: `segments/{hash}.seg`,
`values/{hash}.val`, and `manifests/{op:020}-{hash}.manifest`. A file is never rewritten.
- Source: common technique.
- Verified by: the name matches the hash of the content; no write to an existing file appears in the
  trace.

**S1-CKPT-5 Writes continue during a checkpoint.** The checkpoint reads either a frozen copy of the
graph (`freeze_clone`) or the rows changed since the last checkpoint.
- Source: this project (UltraGraph's `freeze_clone`).
- Verified by: writes acknowledged during a checkpoint are present after recovery, through the log
  if not in the segments.

**S1-CKPT-6 Deterministic compaction.** Merging the same input segments yields byte-identical output.
- Source: TigerBeetle (`data_file.md`: deterministic execution gives byte-identical storage across
  replicas).
- Verified by: compacting the same inputs twice gives equal hashes.

## 11. Recovery restores a prefix that contains every acknowledged batch

**S1-RECOV-1 Opening.**
1. Read both superblocks (S1-SB-3).
2. Load the manifest's segments into a frozen graph and turn it Live.
3. Replay the log records after the checkpoint record. Check the hash chain from the checkpoint
   record onward, and apply the repair rule at the end (S1-REC-3).

- Source: this design.
- Verified by: open after a clean shutdown and after each crash kind yields the expected digest.

**S1-RECOV-2 The prefix property.** Recovery yields the state after some prefix of the submitted
batches, and that prefix contains every acknowledged batch.
- Source: Apache Iggy (its fault sweeps assert that acknowledged history survives).
- Verified by: the fault sweep (S1-SIM-7) against the shadow model (S1-SIM-8).

**S1-RECOV-3 Removing leftovers.** After opening, the engine removes temporary superblock files, log
segments wholly before the checkpoint record, and segment files the current manifest does not
reference. A failed removal is retried at the next open.
- Source: Apache Iggy (its test
  `obsolete_wal_generations_are_reclaimed_after_restart_and_failed_unlink`).
- Verified by: leftovers planted before open are gone afterwards; a removal failed by fault injection
  completes at the next open.

## 12. Reads see applied batches, and snapshots are immutable

**S1-READ-1 Reads through the engine.** A read through the engine sees every applied batch and is
ordered with the writes.
- Source: this design.
- Verified by: a read after an acknowledged write returns it.

**S1-READ-2 Snapshots.** A snapshot read returns an immutable view of the graph at operation N: a
frozen copy shared by reference, which analytics read on other threads.
- Source: GraphOne (a view composed of a snapshot and the log after it); this project
  (`freeze_clone`).
- Verified by: writes after the snapshot do not appear in it; its digest equals the engine's digest
  at N.

## 13. Transactions are atomic, checked in apply, and serializable

**S1-TX-1 Transactions inside a batch.** A transaction is an ordered list of mutations and checks
from one caller. A batch record (S1-REC-1) holds whole transactions, and group commit puts many
transactions into one batch. Apply applies a transaction completely or not at all. When any mutation
or check in it is refused, none of its mutations apply, and the transactions before and after it in
the batch are unaffected.
- Source: DeepBrain (`Ontology::apply` commits one mutation as one transaction); this design
  (transactions inside a group-commit batch).
- Verified by: a batch whose middle transaction fails a check applies the first and the last
  transaction and none of the middle one's mutations; replay gives the same result.

**S1-TX-2 Checks in apply.** A transaction may carry checks, evaluated against the state at its
position in the log before its mutations apply:
- an identity is absent or present;
- an edge is absent;
- an index key's set has at most n members;
- an invariant the host declares.

A failed check refuses the transaction (S1-APP-3). Apply is serial, so no other transaction can come
between a check and the writes that follow it. Snapshot isolation needs a guard key and retries on
conflict to hold such an invariant; this store needs neither.
- **Host invariants.** The engine is generic over a host-supplied invariant trait, statically
  dispatched. Each implementation is pure and deterministic over the state (S1-APP-2). DeepBrain's
  invariants:
  - at most one preferred rank among a subject's values of one kind;
  - at most one absence marker per subject and kind;
  - a software agent keeps an open delegation;
  - a post belongs to exactly one organisation;
  - an identity is unique;
  - a node's group closure holds at most 16 codes.
- Source: DeepBrain (its invariant catalogue, each invariant with the scope over which concurrent
  changes must conflict).
- Verified by: two transactions that each keep an invariant alone but break it together, submitted
  concurrently under random seeds: exactly one commits, in either order. Each named invariant has a
  test that breaks it and is refused.

**S1-TX-3 Upsert.** In one step, a transaction can create a node if its identity is absent, or else
receive the existing node.
- Source: DeepBrain (one node per identity, created by upsert).
- Verified by: sixteen concurrent upserts of one identity create one node, and every caller receives
  its identifier.

**S1-TX-4 Composing writes.** A layer above the store can append mutations and checks to a caller's
transaction before it is submitted. They commit or fail together.
- Source: DeepBrain (`Extensible`).
- Verified by: S1-TX-1.

**S1-TX-5 The commit result.** A committed transaction returns its operation number, which is also
its position in the change feed (S1-FEED-1). A refused transaction returns the check or mutation that
refused it.
- Source: DeepBrain (`Committed`; the feed cursor); this design.
- Verified by: operation numbers rise in commit order; a refusal names the failing item.

**S1-TX-6 Exclusive operations are operations in the log.** These are operations in the log:
- changing a declaration (S1-DECL-1);
- adding or dropping an index (S1-IDX-6);
- reorganising groups, which rewrites the closure of every member of a moved group.

Each takes effect at its position: transactions before it see the old state, and transactions after
it see the new. No pause, drain or repair is needed.
- Source: DeepBrain (reorganisations, index changes and declaration changes as exclusive
  operations); this design (ordering them in the log).
- Verified by:
  - a reorganisation interleaved with concurrent writes in the simulator leaves every member's
    closure consistent with the group tree at every position;
  - the cost of reorganising n members is measured (S1-MEAS-5).

## 14. Indexes are maintained in apply

**S1-IDX-1 Index declarations.** Indexes are declared over values that a host-supplied extractor
trait takes from node and edge payloads, statically dispatched as `EdgeKey` is. An extractor returns
zero or more keys per entity, so a field holding a list yields one key per element.
- Source: DeepBrain (the group closure is a list of codes; periods and cells are lists of
  integers); this project (`EdgeKey`).
- Verified by: extractors returning zero, one and many keys; an entity whose keys change moves
  between index entries.

**S1-IDX-2 Unique index.** A unique index maps a key to at most one entity. S1-TX-2 and S1-TX-3 read
it, and natural keys use it (S1-ID-5).
- Source: DeepBrain (identity is unique across the namespace).
- Verified by: a second entity with the same key is refused; removing the first frees the key.

**S1-IDX-3 Equality index.** An equality index maps a key to the set of entities carrying it, and
reports the set's size without reading the set. Reading a key costs in proportion to the answer.
- **Uses:**
  - a group closure code leading to its nodes ("everything in a group");
  - a kind leading to its nodes;
  - a composite of code and most specific kind leading to its nodes ("a kind within a group", in
    one lookup).
- Source: DeepBrain (its closure, scope and kind keys; counts per group).
- Verified by: after every step of a random workload, the index equals a model rebuilt from
  scratch; every reported size equals its set's length.

**S1-IDX-4 Ordered index.** An ordered index maps integer keys, in order, to entities, and answers
ranges at a cost proportional to the answer. It serves validity start times, time periods and S2
cells; their nested numbering makes the descendants of a cell or a period one range.
- Source: DeepBrain (periods and cells as nested ranges; reads rooted on the time axis).
- Verified by: random ranges against a model; a parent's range holds exactly its descendants'
  keys.

**S1-IDX-5 Indexes match the applied prefix.** Index entries change in the same apply step as their
entity. A read at operation N, snapshots included (S1-READ-2), sees every index exactly as at N. An
index read never misses a commit below its position and never sees one above it.
- Source: this design. DeepBrain's measurements of its current backend recorded index reads that
  missed commits below their start under concurrent writes.
- Verified by: under concurrent writes in the simulator, every index read equals the model at its
  position.

**S1-IDX-6 Index changes.** Adding an index builds it deterministically from the state at its
position in the log; dropping one removes it at its position (S1-TX-6).
- Source: DeepBrain (index changes as exclusive operations); this design.
- Verified by: an index added at N equals, at every later position, the index of a store that
  declared it from the start.

**S1-IDX-7 Indexes survive recovery.** Indexes are written into checkpoint segments or rebuilt from
the loaded graph at recovery (§21). Either way, a recovered index equals the index before the crash
at the recovered position.
- Source: this design.
- Verified by: the fault sweep (S1-SIM-7) compares index answers with the shadow model as well.

## 15. Queries are lookups, intersections, filters and walks

**S1-QRY-1 Primitives.** The store answers:
- an entity by identifier or unique key, in a bounded number of lookups;
- entities by equality key, or by key range;
- the intersection of such sets;
- a filter over payloads.

Every answer except the bounded lookup costs in proportion to its size.
- Source: DeepBrain (its questions and their cost contracts: with the answer, with the walked set,
  bounded).
- Verified by: each primitive against the model; the entries each one visits are counted and
  compared with its contract.

**S1-QRY-2 Root, then filter.** A query names its root (one primitive) and its filters. The store
evaluates the root and then filters its result, in that order. Choosing the most selective root is
the caller's job; DeepBrain's projection plans it.
- Source: DeepBrain (reads root on the most selective key, in a fixed order).
- Verified by: a query's result equals its root's result filtered, for every root and filter
  pairing in the suite.

**S1-QRY-3 Walks with pruning.** A walk starts from a set of nodes and follows edges of given kinds in
a given direction, up to a depth bound, applying a node filter at every level. A node the filter
removes is not expanded, so nodes reachable only through it are not reached. The cost grows with the
walked set.
- Both directions are supported. A history read roots on the time index (S1-IDX-4) instead of
  walking a reverse list that grows with the subject's history.
- Source: DeepBrain (a walk restricted to groups is pruned at every level, and a node outside the
  group hides the in-group nodes below it).
- Verified by: walks on trees and acyclic graphs against a model, with filters that remove interior
  nodes; the number of visited nodes equals the walked set.

**S1-QRY-4 Absence filters.** A filter can require a field to be absent: an open window has no end,
and a live node has no retraction time.
- Source: DeepBrain (reads exclude retracted nodes and closed windows by the absence of a field).
- Verified by: filters on present, absent and changed fields against the model.

## 16. The log is the change feed

**S1-FEED-1 One total order.** Subscribers receive committed transactions in log order, each once,
with no gaps, from a cursor (the operation number).
- A node's creation precedes every transaction that names it.
- A host's own write is delivered after every transaction that precedes it in the log.
- Source: DeepBrain (one gapless total order per namespace, with no duplicates, respecting causality
  and real time); this design (the log is the feed).
- Verified by: under random workloads, the feed equals the log's committed transactions; resuming
  from any cursor after a restart continues without a gap or a duplicate.

**S1-FEED-2 A snapshot, then the tail.** A subscription returns the state at a position and the
transactions after it, as `ContextStorageStream::subscribe` requires. A cursor older than the oldest
retained log segment is answered from the latest checkpoint.
- Source: this project (`ContextStorageStream`); DeepBrain.
- Verified by: subscriptions at the current position, at old positions, and before the oldest
  retained segment.

**S1-FEED-3 What a feed entry carries.** A feed entry carries the transaction's mutations and the
declaration version it was checked against. Personal values travel by reference (S1-PII-1).
- Source: DeepBrain (feed records carry personal values by reference, never as copies).
- Verified by: S1-PII-1's byte scan covers the feed.

## 17. Declarations, namespaces, retraction and personal data

**S1-DECL-1 Declarations are store state.** The vocabulary (kinds, relation kinds, extensions) and
the group tree are entities of the store. Apply validates each mutation against the declarations in
force at its position, so a write never commits against a declaration that was replaced before it.
- Source: DeepBrain (writes check their kinds against the declarations; an extension upgrade changes
  them); this design.
- Verified by: a declaration change and a write against the old declaration, in both orders: the
  write commits only when it comes first.

**S1-NS-1 Namespaces.** One engine serves one namespace: one tenant, one graph. A process may open
several engines, each with its own directory.
- Source: DeepBrain (a namespace holds one graph and separates customers).
- Verified by: two engines in one process share no state, and a crash recovers each from its own
  directory.

**S1-RET-1 Retraction keeps history.** Retracting a subject sets its retraction time and closes its
windows, and the node and its edges stay readable for history reads. Physical removal (S1-APP-4) is a
separate operation.
- Source: DeepBrain (a subject's history is its closed windows and retracted nodes, read by time).
- Verified by: a retracted node appears in history reads, and absence filters (S1-QRY-4) exclude it
  from current reads.

**S1-PII-1 Personal values stay out of the immutable files.** Values the schema marks as personal go
to an erasable value store. Log records, segments, feed entries and backups carry only a reference
to them.
- Source: DeepBrain (a person's name lives only in the personal fields of one node, and the feed
  carries them by reference); this design (keeping them out of the immutable log).
- Verified by: after a personal value is written, a byte scan of every log segment, segment file and
  feed entry finds no copy of it.

**S1-PII-2 Erasure.** Erasing a person's personal values deletes them from the erasable value store,
and every reference then resolves to "erased". Replay and recovery reproduce the erased state.
- Source: DeepBrain (erasure of the personal fields of one node).
- Verified by: erase, crash, recover: the values stay erased, and their bytes appear nowhere in the
  store directory.

## 18. The storage interface has a std implementation and a simulated one

**S1-STOR-1 The interface.** The interface is asynchronous and offers:
- files: open or create, read at an offset, write at an offset, length, truncate, sync;
- directories: create, sync, rename, remove a file, list.

It holds only the operations stage 1 uses.
- Source: Apache Iggy (the shape of `DurableStorage` and `DurableFile`,
  `core/journal/src/durable_storage.rs`).
- Verified by: the same contract tests pass on both implementations.

**S1-STOR-2 The std implementation.** It uses positional reads and writes, and `File::sync_data` for
file syncs. On Apple platforms that call issues `F_FULLFSYNC`, measured at about 5.5 ms per barrier on
the benchmark machine (`PersistentStore.md`, Appendix A). A directory sync syncs an open handle on the
directory.
- Source: Rust standard library (`library/std/src/sys/fs/unix.rs`).
- Verified by: contract tests on a temporary directory. Directory sync on macOS is an open question
  (§21).

**S1-SIM-1 Two copies of every file and directory.** In the simulated filesystem, every file holds
buffered and stable bytes, and every directory holds current and stable entries.
- A write changes buffered bytes.
- A file sync copies that file's buffered bytes to stable.
- A directory sync copies that directory's entries to stable.
- Source: Apache Iggy (`core/simulator/src/storage.rs`).
- Verified by: the model's own tests, including that a file sync does not make the file's directory
  entry durable.

**S1-SIM-2 Two kinds of crash.** A process crash keeps buffered bytes and current entries, as the OS
cache survives it. A power loss reverts every file and directory to its stable copy. Handles opened
before a crash fail after it.
- Source: Apache Iggy (same file).
- Verified by: the model's tests for each crash kind and for stale handles.

**S1-SIM-3 Faults at an operation.** A fault targets the i-th storage operation and takes one of four
forms:
- fail before the operation;
- fail after it;
- a torn write reported as an error;
- a torn write reported as success.

- Source: Apache Iggy (same file).
- Verified by: each form at an operation of each kind.

**S1-SIM-4 Writeback and interleaving.**
- Writeback can make later pages stable while earlier pages stay dirty.
- A writeback failure is reported to the next sync of a handle opened before it, following Linux
  writeback-error semantics.
- Writes or syncs can be paused to force an interleaving.
- Source: Apache Iggy (same file).
- Verified by: a recovery that trusts a prefix of its log fails under unordered writeback; the engine
  passes.

**S1-SIM-5 Faults beyond the reference model.** The model adds three faults:
- latent sector errors: stable bytes that read back unreadable or altered;
- silent corruption of stable bytes;
- misdirected writes: bytes land at a wrong offset or in a wrong file.

- Source: TigerBeetle (Safety names latent sector errors, silent corruption and "misdirect IO" among
  the failures it handles). Iggy's simulator model at `fe3b4269` does not inject these three.
- Verified by: every checksum in S1-REC-1, S1-SB-1 and S1-CKPT-4 rejects each fault; no corrupted
  byte reaches the graph.

**S1-SIM-6 The deterministic executor.** The executor is single-threaded. When several tasks are
ready, it picks one uniformly at random from a seeded generator. Time advances only when the
simulation advances it. In stage 1 it runs the engine's group-commit, checkpoint and reader tasks.
- Source: Apache Iggy (`core/simulator/src/executor`).
- Verified by: the same seed gives the same interleaving and the same trace; different seeds give
  different interleavings.

**S1-SIM-7 Fault sweeps.**
1. Run a scenario on the simulator and record its trace of storage operations.
2. For every operation index and every fault of S1-SIM-3 and S1-SIM-5, rerun the scenario with that
   fault at that index.
3. Crash it with each kind of crash, recover it, and check S1-RECOV-2.

- Source: Apache Iggy (`core/simulator/src/storage/tests.rs`, for example
  `wal_fault_sweep_preserves_acknowledged_history_at_every_io_boundary`).
- Verified by: the sweep passes over a commit, a segment roll, a checkpoint and a recovery that is
  itself interrupted.

**S1-SIM-8 Shadow model and oracle.** A shadow model applies every submitted batch to a plain,
map-based graph that shares no code with the engine's apply. After recovery, the engine's digest must
equal the model's digest after the recovered prefix.
- Source: Apache Iggy (`core/simulator/src/workload/shadow.rs` and `oracle.rs`); this project (the
  TDD protocol's requirement that expected values come from an independent source).
- Verified by: S1-SIM-7.

**S1-SIM-9 Reproducibility.** Every simulator run is reproducible from one seed, and a failing run
reports its seed.
- Source: Apache Iggy (the seeded executor).
- Verified by: rerunning a reported seed reproduces the failure.

## 19. The conformance kit and the measurements decide

**S1-MEAS-1 Apply throughput.** Mutations applied per second on one core, per operation kind, at
10^3 to 10^8 edges.
- Source: this design.

**S1-MEAS-2 Durable commit throughput and latency against group size,** on the benchmark machine and
on Linux NVMe.
- Source: this design.

**S1-MEAS-3 Recovery and checkpoint costs:**
- recovery time per gigabyte (loading segments and replaying the log);
- checkpoint duration;
- the checkpoint's effect on write latency.
- Source: this design.

**S1-MEAS-4 DeepBrain's targets.** On the benchmark machine, the store beats the figures DeepBrain
measured for its current backend, Dgraph v25.4.0, standalone in Docker with 8 CPUs and 8 GB
(2026-10-12):
- identity upserts creating one node each, with 16 writers: 2,488 transactions per second;
- writes of a 25-field envelope: about 130,000 quads per second on the hottest field;
- "a kind within a group": 2.6 ms over 10,000 nodes, 21 ms over 100,000;
- a group filter inside a walk: 0.4 ms over 50 walked nodes, 12 ms over 5,000;
- a recursion pruned at every level, over 1,110 nodes and returning 155: 0.79 ms.

Each comparison states the incumbent's container limits, and the durability and isolation of both
stores. Dgraph provides snapshot isolation.
- Source: DeepBrain (its measurements of its current backend).

**S1-MEAS-5 The cost of exclusive operations.** Measure the time, and the stall of the apply thread,
for reorganising a group of n members and for building an index over n entities.
- Source: this design.

**S1-CONF-1 Conformance first.** Through a DeepBrain adapter, the store passes DeepBrain's
conformance kit: for every question, it gives the reference evaluator's answer. No measurement of
S1-MEAS-4 counts until it does.
- Source: DeepBrain (its reference evaluator and conformance kit, which every backend passes).

Every timing names its machine; the benchmark machine is an Apple M3 Max with 16 cores and 128 GB.
These results decide whether stages 4 and 5 are needed (`PersistentStore.md` §8.8).

## 20. TDD order and the defects the suite must catch

The five phases of `openspec/specs/unified-math-tdd-protocol/spec.md` apply:
1. The API compiles with `unimplemented!()` bodies.
2. The suite, written from the identifiers above, is observed failing.
3. A defect audit.
4. Implementation.
5. Mutation testing.

Phase 3 injects at least the following defects, and the suite must reject each:

| Defect | Requirement it breaks |
|---|---|
| Acknowledging before the barrier returns | S1-LOG-5 |
| Acknowledging part of a group whose barrier failed | S1-LOG-5 |
| Retrying a failed sync and then acknowledging | S1-LOG-5 |
| Skipping the directory sync after creating a segment | S1-LOG-3 |
| Applying a batch before it is durable | S1-APP-1 |
| Acknowledging group k + 1 before group k while overlapping barriers | S1-ARCH-4 |
| Spawning a thread outside the host's spawner, or setting core affinity | S1-ARCH-4 |
| Iterating a hash map inside apply | S1-APP-2 |
| Reading the wall clock inside apply | S1-APP-2 |
| Writing the superblock in place | S1-SB-2 |
| Reading through a superblock copy that fails verification | S1-SB-3 |
| Deleting log segments before the superblock is durable | S1-CKPT-2 |
| Referencing a segment before `segments/` is synced | S1-CKPT-2 |
| Truncating damage that has a valid record behind it | S1-REC-3 |
| Accepting a record with a broken hash chain | S1-REC-1 |
| Treating an insert with an existing identifier as an update | S1-ID-4 |
| Reissuing a reserved index after a restart | S1-ID-3 |
| Applying part of a refused transaction | S1-TX-1 |
| A refused transaction affecting its neighbours in the batch | S1-TX-1 |
| Evaluating a check against a state other than its position in the log | S1-TX-2 |
| Updating an index in a step separate from its entity | S1-IDX-5 |
| An index read that misses a commit below its position | S1-IDX-5 |
| A walk that expands a node its filter removed | S1-QRY-3 |
| A feed with a gap, a duplicate or a reordered entry | S1-FEED-1 |
| A write that commits against a replaced declaration | S1-DECL-1 |
| A personal value written into the log, a segment or a feed entry | S1-PII-1 |
| A logical retraction that removes the node | S1-RET-1 |

## 21. Open questions

1. The slot width b (proposed 10) and the mutation record size (proposed 48 bytes).
2. The record checksum and the content hash, and with them the new repository's dependency policy.
3. Holding the whole graph in memory in stage 1 (proposed: yes; graphs larger than memory are
   stage 4).
4. Directory sync with `F_FULLFSYNC` on macOS.
5. The checkpoint trigger: an operation count, a byte count in the log, or both.
6. The facade's API, and whether stage 1 implements `ContextStorage` directly.
7. Limits on payload size inside a batch's value area.
8. The ring's capacity, and whether its channels come from a crate or from the standard library.
9. Indexes written into checkpoint segments, or rebuilt from the loaded graph at recovery (S1-IDX-7).
10. The erasable value store: personal values in files erased in place, or encrypted under a key per
    person whose deletion erases them (S1-PII-2).
11. Whether reorganising a large group applies as one operation, or in bounded steps under a
    declaration version (S1-TX-6, S1-MEAS-5).
12. Reads of validity at an instant. DeepBrain's own design asks whether the keys of such a read grow
    with the years since the earliest open window; if they do, the ordered index needs a form for
    open windows that keeps them bounded (S1-IDX-4).
