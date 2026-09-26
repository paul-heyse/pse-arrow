"""Two small, source-extracted ChEDL numerical functions for bounded probes.

NOT an installation/import of thermo or chemicals. The numerical bodies below
were transcribed from pinned connector source excerpts. Upstream docstrings and
unrelated package code were omitted. math.exp replaces the module import context.
No package constructor, flash dispatcher, database, compiled dependency, or
upstream test suite is included or exercised by importing this file.

Sources:
thermo/nrtl.py, NRTL_gammas, commit
2bb466e98439c2395a004e7095d07f9b97a5a0f0; blob
816d8eb7034bfc553b9d651acbb8dfeba9432cbf; numerical body retrieved in source window 2010-2200.
The source window is a locator, not a claim that this extracted module is a
byte-for-byte copy of the original file.
chemicals/rachford_rice.py, Rachford_Rice_flash_error, commit
e79047588b30cfabc564c79fb26d760c746877d7; blob
ef08efe5244f742525e7b914ac95736219febd9f; fetched lines 610-735.

Copyright (C) 2016, 2017, 2018, 2019, 2020, 2021 Caleb Bell
<Caleb.Andrew.Bell@gmail.com> (NRTL).
Copyright (C) 2016, 2017, 2018, 2019, 2020 Caleb Bell
<Caleb.Andrew.Bell@gmail.com> (Rachford-Rice).

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
"""
from math import exp


def NRTL_gammas(xs, taus, alphas):
    gammas = []
    cmps = range(len(xs))
    # Gs does not depend on composition
    Gs = []
    for i in cmps:
        alphasi = alphas[i]
        tausi = taus[i]
        Gs.append([exp(-alphasi[j]*tausi[j]) for j in cmps])

    td2s = []
    tn3s = []
    for j in cmps:
        td2 = 0.0
        tn3 = 0.0
        for k in cmps:
            xkGkj = xs[k]*Gs[k][j]
            td2 += xkGkj
            tn3 += xkGkj*taus[k][j]
        td2 = 1.0/td2
        td2xj = td2*xs[j]
        td2s.append(td2xj)
        tn3s.append(tn3*td2*td2xj)

    for i in cmps:
        tn1, td1, total2 = 0., 0., 0.
        Gsi = Gs[i]
        tausi = taus[i]
        for j in cmps:
            xjGji = xs[j]*Gs[j][i]
            td1 += xjGji
            tn1 += xjGji*taus[j][i]
            total2 += Gsi[j]*(tausi[j]*td2s[j] - tn3s[j])
        gamma = exp(tn1/td1 + total2)
        gammas.append(gamma)
    return gammas


def Rachford_Rice_flash_error(V_over_F: float, zs: list[float], Ks: list[float]) -> float:
    err = 0.0
    for i in range(len(zs)):
        err += zs[i]*(Ks[i] - 1.0)/(1.0 + V_over_F*(Ks[i] - 1.0))
    return err
