using System;
class PolicyTests {
 static void Check(bool x){if(!x)throw new Exception("policy assertion");}
 static void Main(){
 Check(TemperaturePolicy.IsCore("P-Core #1"));Check(TemperaturePolicy.IsCore("E-Core #3"));Check(TemperaturePolicy.IsCore("CPU Core #2"));
 foreach(var n in new[]{"CPU Package","Core Max","Core Average","P-Core #1 Distance to TjMax","Core #1 (Tctl)","CPU CCD1 (Tdie)"})Check(!TemperaturePolicy.IsCore(n));
 var a=TemperaturePolicy.Range(new[]{37.0,61.0,double.NaN,1000.0});Check(a[0]==37&&a[1]==61);
 var b=TemperaturePolicy.Range(new[]{44.0,45.0});Check(b[0]==44&&b[1]==45);
 Check(TemperaturePolicy.Range(new double[0])==null);
 Console.WriteLine("TEMPERATURE_POLICY_OK");
 }
}
