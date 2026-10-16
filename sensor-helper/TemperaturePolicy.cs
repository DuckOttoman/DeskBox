using System;
using System.Collections.Generic;
using System.Text.RegularExpressions;
public static class TemperaturePolicy {
 public static bool IsCore(string name){return Regex.IsMatch(name ?? "", @"^(?:(?:CPU |P-|E-)?Core) #?[0-9]+$",RegexOptions.CultureInvariant);}
 public static double[] Range(IEnumerable<double> values){double lo=double.PositiveInfinity,hi=double.NegativeInfinity;foreach(var v in values){if(double.IsNaN(v)||double.IsInfinity(v)||v < -20||v>150)continue;lo=Math.Min(lo,v);hi=Math.Max(hi,v);}return double.IsInfinity(lo)?null:new[]{lo,hi};}
}
