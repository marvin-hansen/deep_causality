# Chronometric GM Recovery

This example recovers Earth's gravitational parameter from one week of Galileo
satellite clock data. The only physics input is time-dilation differences
between satellites at different altitudes.

## The Result

One full GPS week of broadcast clocks from Galileo satellite E14, fed through
the J2-corrected weak-field 1PN inversion, produces this:

```text
                       Recovered          Reference (JGM-3 / IERS 2010)
  GM_Earth  [m³/s²]    3.994180e14        3.986004e14
  M_Earth   [kg]       5.984418e24        5.972190e24

  Relative error (GM):   0.2051 %   (2.051e-3)
  Relative error (M):    0.2047 %   (2.047e-3)
```

That is Earth's mass, weighed by satellite clocks accurate to 0.2%.
Two important things that are worth mentioning. First, an analytical inversion of
the relativistic clock equation worked at high precision despite a small dataset.
Second, the bind-chain composition, wrapping the kernel into a typed pipeline you can
compose further with either causal monads or causaloids.

## Running It

```bash
cargo run -p chronometric_examples --example gm_recovery --release
```

Real Galileo broadcast clock and SP3 orbit data ship with the example: one
full GPS week, satellite E14. No external downloads, no dependencies beyond
the workspace.

## The Pipeline

`main.rs` shows the structural showcase up front. Five stages, each returning
a `PropagatingEffect`, composed through `.bind`:

```rust
let result = PropagatingEffect::pure(inputs)
    .bind(stage_load::<FloatType>)
    .bind(stage_align)
    .bind(stage_pair)
    .bind(stage_solve_gm)
    .bind(stage_aggregate);
```

What each stage does, in order:

1. **load**. Reads `.clk` and `.sp3` files for a single satellite across all
   bundled GPS-week datasets, concatenates them, sorts by timestamp.
2. **align**. Runs a 10th-order Lagrange interpolation on the orbit data to
   match clock timestamps, then restores the IGS-removed periodic
   relativistic correction. Output: a vector of `SpaceTimeCoordinate` samples.
3. **pair**. Slides a window across the coordinate vector, picking pairs
   separated by roughly 50 minutes of orbital phase. The sliding scheme
   matches chronometric-geodesy convention and avoids the all-pairs failure
   mode where every pair ends up anchored to the first few coordinates.
4. **solve_gm**. Applies the J2-corrected 1PN kernel (`solve_gm_analytical`)
   from `deep_causality_physics::chronometric` to each pair.
5. **aggregate**. Filters the per-pair estimates through a Median Absolute
   Deviation outlier rejection, then reduces to mean, median, standard
   deviation. Earth's mass is derived as $M = GM / G$.

Each stage is generic over the floating-point type. The default is
`Float106` (double-double, around 32 decimal digits); switching to `f64` is
a one-line change to the `FloatType` alias in `main.rs`.

## Data Layout

```text
data/gnss/
├── gbm18770.clk + gbm18770.sp3
├── gbm18771.clk + gbm18771.sp3
├── gbm18772.clk + gbm18772.sp3
├── gbm18773.clk + gbm18773.sp3
├── gbm18774.clk + gbm18774.sp3
├── gbm18775.clk + gbm18775.sp3
└── gbm18776.clk + gbm18776.sp3
```

GPS week 1877, days 0 through 6, from 2016. The `.clk` files are RINEX 3
clock products from the GFZ Multi-GNSS analysis center; the `.sp3` files
are precise satellite orbits in standard SP3 format. Both are parsed by
the local `data_loader` module.

## The Math, Briefly

The kernel inverts the relativistic clock equation in the weak-field 1PN
limit:

$$\dot\tau = 1 + \frac{\Phi(r,\theta)}{c^2} - \frac{v^2}{2c^2}$$

with the J2-corrected Earth potential

$$\Phi(r,\theta) = -\frac{GM}{r}\left[1 - J_2 \left(\frac{R_{eq}}{r}\right)^2 \frac{3\cos^2\theta - 1}{2}\right].$$

Given two `SpaceTimeCoordinate` samples with measured clock-drift rates,
$GM$ falls out algebraically:

$$GM = \frac{c^2(\dot\tau_b - \dot\tau_a) + \tfrac{1}{2}(v_b^2 - v_a^2)}{1/r_{\text{eff},a} - 1/r_{\text{eff},b}}.$$

Bjerhammar (1975) and Vermeer (1983) established this kind of inversion as
the foundation of chronometric geodesy. The kernel implements it directly,
generic over any precision type that satisfies `RealField + From<f64>`. See
the [chronometric kernel
documentation](../../deep_causality_physics/src/chronometric/) for the full
assumption envelope and the regimes where the method stops working.

## Scope and Limitations

This is a public demonstration of the kernel, not a production-grade GM
determination. With one week of one satellite, accuracy tops out around the
per-mille level. The full multi-year, multi-satellite analysis at the
Center for Causal Dynamics reaches sub-ppm.

A few simplifications worth knowing about:

- The Lagrange interpolation does not handle SP3 boundary discontinuities
  specially. Most Galileo SP3 products are continuous across day
  boundaries, so the simplification holds for the bundled data.
- The MAD outlier filter runs once over per-pair estimates. Iterative
  refinement and per-orbit outlier classification are intentionally omitted
  to keep the example readable.
- Pair construction uses a fixed-window sliding scheme. See `pipeline.rs`
  for the window-size and step constants if you want to tune it.

What the example does demonstrate well: the framework's bind-chain composes
a real physical-inverse problem end-to-end, and the chronometric kernel
recovers published JGM-3 reference values on real data.

## Acknowledgments

This example was contributed by the Center for Dynamic Causality. It is a
smaller, public replication of a larger experiment covering multiple years of GNSS data of the 
Galileo constellation. The complete experiment is publicly available at:

https://github.com/causalcenter/chronodynamics

## References

1. Bjerhammar, A. (1975). *Discrete approaches to the solution of the
   boundary value problem in physical geodesy*. Bulletin Géodésique 49,
   23–35.
2. Vermeer, M. (1983). *Chronometric levelling*. Reports of the Finnish
   Geodetic Institute, 83:2.
3. Petit, G., Luzum, B. (eds.) (2010). *IERS Conventions (2010)*. IERS
   Technical Note No. 36.
4. NASA Goddard Space Flight Center (1996). *JGM-3 Earth Gravity Model*.

## License

MIT. See the workspace `LICENSE` file.
