using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.IO.Pipes;
using System.Linq;
using System.Security.AccessControl;
using System.Security.Principal;
using System.ServiceProcess;
using System.Threading;
using System.Web.Script.Serialization;
using LibreHardwareMonitor.Hardware;

class SensorService : ServiceBase {
 public SensorService(){ServiceName=Program.ServiceName;CanStop=true;}
 protected override void OnStart(string[] args){new Thread(()=>{Program.Serve();Stop();}){IsBackground=true}.Start();}
 protected override void OnStop(){Program.Stopping=true;}
}
static class Program {
 internal const string ServiceName="DeskBoxCpuSensor";
 const string PipeName="DeskBoxCpuSensor.v1";
 internal static volatile bool Stopping;
 static readonly JavaScriptSerializer Json=new JavaScriptSerializer();
 static string snapshot="{\"status\":\"starting\",\"timestamp\":0,\"cores\":[]}";
 static long lastClient=DateTime.UtcNow.Ticks;
 static string ProtectedDir {get{return Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles),"DeskBox Sensor");}}
 static bool Admin(){return new WindowsPrincipal(WindowsIdentity.GetCurrent()).IsInRole(WindowsBuiltInRole.Administrator);}
 static int Run(string file,string args){using(var p=Process.Start(new ProcessStartInfo(file,args){UseShellExecute=false,CreateNoWindow=true,WindowStyle=ProcessWindowStyle.Hidden})){if(!p.WaitForExit(60000))throw new Exception("Component operation timed out");return p.ExitCode;}}
 static string Sc {get{return Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.System),"sc.exe");}}
 static bool HasPawn(){using(var k=Microsoft.Win32.RegistryKey.OpenBaseKey(Microsoft.Win32.RegistryHive.LocalMachine,Microsoft.Win32.RegistryView.Registry64).OpenSubKey(@"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\PawnIO")){return k!=null;}}
 static string[] Payload(string dir){return Dependencies.Hashes.Keys.Concat(new[]{"DeskBox.Sensor.exe","dependencies.json"}).Select(n=>Path.Combine(dir,n)).Where(File.Exists).ToArray();}
 static void ValidateDependencies(){foreach(var item in Dependencies.Hashes){var file=Path.Combine(AppDomain.CurrentDomain.BaseDirectory,item.Key);using(var sha=System.Security.Cryptography.SHA256.Create())using(var stream=File.OpenRead(file)){string hash=BitConverter.ToString(sha.ComputeHash(stream)).Replace("-","").ToLowerInvariant();if(hash!=item.Value)throw new Exception("Sensor component verification failed: "+item.Key);}}}
 static void SafeDirectory(string path){for(var d=new DirectoryInfo(path);d!=null;d=d.Parent)if(d.Exists&&(d.Attributes&FileAttributes.ReparsePoint)!=0)throw new Exception("Sensor install path must not contain junctions");}
 static string HashFile(string path){using(var sha=System.Security.Cryptography.SHA256.Create())using(var stream=File.OpenRead(path))return BitConverter.ToString(sha.ComputeHash(stream)).Replace("-","").ToLowerInvariant();}
 static void Stage(string expectedSelf){
  if(!Admin())throw new Exception("Administrator authorization required");
  if(expectedSelf==null||!System.Text.RegularExpressions.Regex.IsMatch(expectedSelf,"^[a-fA-F0-9]{64}$"))throw new Exception("Expected collector digest required");
  SafeDirectory(ProtectedDir);Directory.CreateDirectory(ProtectedDir);
  var acl=new DirectorySecurity();acl.SetAccessRuleProtection(true,false);
  foreach(var sid in new[]{WellKnownSidType.LocalSystemSid,WellKnownSidType.BuiltinAdministratorsSid})acl.AddAccessRule(new FileSystemAccessRule(new SecurityIdentifier(sid,null),FileSystemRights.FullControl,InheritanceFlags.ContainerInherit|InheritanceFlags.ObjectInherit,PropagationFlags.None,AccessControlType.Allow));
  acl.AddAccessRule(new FileSystemAccessRule(new SecurityIdentifier(WellKnownSidType.BuiltinUsersSid,null),FileSystemRights.ReadAndExecute,InheritanceFlags.ContainerInherit|InheritanceFlags.ObjectInherit,PropagationFlags.None,AccessControlType.Allow));Directory.SetAccessControl(ProtectedDir,acl);
  string stage=Path.Combine(ProtectedDir,"stage-"+Guid.NewGuid().ToString("N"));Directory.CreateDirectory(stage);
  var hashes=new Dictionary<string,string>(Dependencies.Hashes);hashes.Add("DeskBox.Sensor.exe",expectedSelf.ToLowerInvariant());
  try{
   foreach(var item in hashes){
    string source=Path.Combine(AppDomain.CurrentDomain.BaseDirectory,item.Key),dest=Path.Combine(stage,item.Key);
    using(var input=new FileStream(source,FileMode.Open,FileAccess.Read,FileShare.Read))using(var output=new FileStream(dest,FileMode.CreateNew,FileAccess.Write,FileShare.None))input.CopyTo(output);
    if(HashFile(dest)!=item.Value)throw new Exception("Staged sensor component verification failed: "+item.Key);
   }
   using(var sc=new ServiceController(ServiceName))try{if(sc.Status!=ServiceControllerStatus.Stopped){sc.Stop();sc.WaitForStatus(ServiceControllerStatus.Stopped,TimeSpan.FromSeconds(15));}}catch(InvalidOperationException){}
   foreach(var process in Process.GetProcessesByName("DeskBox.Sensor")){using(process){try{if(process.Id!=Process.GetCurrentProcess().Id&&String.Equals(process.MainModule.FileName,Path.Combine(ProtectedDir,"DeskBox.Sensor.exe"),StringComparison.OrdinalIgnoreCase)){process.Kill();if(!process.WaitForExit(5000))throw new Exception("Previous collector is still stopping");}}catch(InvalidOperationException){}}}
   foreach(var item in hashes){string dest=Path.Combine(ProtectedDir,item.Key);if(File.Exists(dest)&&(File.GetAttributes(dest)&FileAttributes.ReparsePoint)!=0)throw new Exception("Invalid component link");if(!File.Exists(dest)||HashFile(dest)!=item.Value)File.Copy(Path.Combine(stage,item.Key),dest,true);}
  }finally{foreach(var f in Directory.GetFiles(stage))File.Delete(f);Directory.Delete(stage);}
 }
 static void EnsureDriver(){if(!HasPawn()){if(Run(Path.Combine(ProtectedDir,"PawnIO_setup.exe"),"/S")!=0||!HasPawn())throw new Exception("PawnIO installation incomplete");}}
 static void Install(string expectedSelf){
  Stage(expectedSelf);EnsureDriver();string exe=Path.Combine(ProtectedDir,"DeskBox.Sensor.exe");
  if(Run(Sc,"create "+ServiceName+" binPath= \"\\\""+exe+"\\\" --service\" start= demand DisplayName= \"DeskBox CPU Temperature\"")!=0){
   if(Run(Sc,"config "+ServiceName+" binPath= \"\\\""+exe+"\\\" --service\" start= demand")!=0)throw new Exception("Unable to configure sensor service");
  }
  if(Run(Sc,"sdset "+ServiceName+" D:(A;;CCDCLCSWRPWPDTLOCRSDRCWDWO;;;SY)(A;;CCDCLCSWRPWPDTLOCRSDRCWDWO;;;BA)(A;;CCLCRP;;;BU)")!=0)throw new Exception("Unable to restrict sensor service access");
  using(var sc=new ServiceController(ServiceName)){sc.Start();}
 }
 static void Remove(){
  if(!Admin())throw new Exception("Administrator authorization required");
  using(var sc=new ServiceController(ServiceName))try{if(sc.Status!=ServiceControllerStatus.Stopped){sc.Stop();sc.WaitForStatus(ServiceControllerStatus.Stopped,TimeSpan.FromSeconds(15));}}catch(InvalidOperationException){}
  int result=Run(Sc,"delete "+ServiceName);if(result!=0&&result!=1060)throw new Exception("Unable to remove sensor service");
  SafeDirectory(ProtectedDir);if(Directory.Exists(ProtectedDir))foreach(var p in Payload(ProtectedDir)){if((File.GetAttributes(p)&FileAttributes.ReparsePoint)!=0)throw new Exception("Invalid component link");File.Delete(p);}
  // PawnIO may be shared with other monitoring software and is intentionally retained.
 }
 static object Read(Computer computer){
  var cores=new List<Dictionary<string,object>>();var packages=new List<double>();
  foreach(var hw in computer.Hardware){hw.Update();foreach(var s in hw.Sensors){
   if(s.SensorType!=SensorType.Temperature||!s.Value.HasValue)continue;double v=s.Value.Value;if(TemperaturePolicy.Range(new[]{v})==null)continue;
   if(TemperaturePolicy.IsCore(s.Name))cores.Add(new Dictionary<string,object>{{"id",s.Identifier.ToString()},{"value",v}});
   else if(s.Name=="CPU Package"||s.Name=="Core (Tdie)"||s.Name=="CPU (Tctl/Tdie)")packages.Add(v);
  }}
  return new{status=cores.Count>0?"ok":!HasPawn()?"driver_missing":packages.Count>0?"package_only":"sensor_unavailable",timestamp=DateTimeOffset.UtcNow.ToUnixTimeMilliseconds(),cores=cores,package=packages.Count>0?(double?)packages.Max():null};
 }
 static void Sampling(){
  var computer=new Computer{IsCpuEnabled=true};try{computer.Open();while(!Stopping){
   try{string next=Json.Serialize(Read(computer));Interlocked.Exchange(ref snapshot,next);}catch{Interlocked.Exchange(ref snapshot,"{\"status\":\"read_failed\",\"timestamp\":0,\"cores\":[]}");}
   Thread.Sleep(1000);
  }}
  catch{Interlocked.Exchange(ref snapshot,"{\"status\":\"sensor_unavailable\",\"timestamp\":0,\"cores\":[]}");}finally{computer.Close();}
 }
 internal static void Serve(){
  using(var mutex=new Mutex(false,@"Global\DeskBoxCpuSensor.Collector")){
   if(!mutex.WaitOne(0))return;
   try{
    new Thread(Sampling){IsBackground=true}.Start();
    while(!Stopping&&(DateTime.UtcNow.Ticks-Interlocked.Read(ref lastClient))/TimeSpan.TicksPerSecond<60){
     var security=new PipeSecurity();
     security.AddAccessRule(new PipeAccessRule(new SecurityIdentifier(WellKnownSidType.NetworkSid,null),PipeAccessRights.FullControl,AccessControlType.Deny));
     security.AddAccessRule(new PipeAccessRule(new SecurityIdentifier(WellKnownSidType.LocalSystemSid,null),PipeAccessRights.FullControl,AccessControlType.Allow));
     security.AddAccessRule(new PipeAccessRule(WindowsIdentity.GetCurrent().User,PipeAccessRights.FullControl,AccessControlType.Allow));
     security.AddAccessRule(new PipeAccessRule(new SecurityIdentifier(WellKnownSidType.BuiltinUsersSid,null),PipeAccessRights.Read,AccessControlType.Allow));
     try{using(var pipe=new NamedPipeServerStream(PipeName,PipeDirection.Out,1,PipeTransmissionMode.Byte,PipeOptions.Asynchronous,0,8192,security)){
      var pending=pipe.BeginWaitForConnection(null,null);
      while(!pending.AsyncWaitHandle.WaitOne(500)){if(Stopping||(DateTime.UtcNow.Ticks-Interlocked.Read(ref lastClient))/TimeSpan.TicksPerSecond>=60)return;}
      pipe.EndWaitForConnection(pending);Interlocked.Exchange(ref lastClient,DateTime.UtcNow.Ticks);
      byte[] bytes=System.Text.Encoding.UTF8.GetBytes(Interlocked.CompareExchange(ref snapshot,null,null)+"\n");
      var write=pipe.BeginWrite(bytes,0,bytes.Length,null,null);if(write.AsyncWaitHandle.WaitOne(1000))try{pipe.EndWrite(write);}catch(IOException){}
     }}catch(IOException){}catch(ObjectDisposedException){}
    }
   }finally{Stopping=true;mutex.ReleaseMutex();}
  }
 }
 static void Probe(){var c=new Computer{IsCpuEnabled=true};try{c.Open();for(int i=0;i<4;i++){Console.WriteLine(Json.Serialize(Read(c)));Thread.Sleep(1000);}}finally{c.Close();}}
 [STAThread] static int Main(string[] args){
  try{
   string mode=args.Length>0?args[0]:"";
   if(mode=="--service"){ServiceBase.Run(new SensorService());return 0;}
   if(mode=="--install"){Install(args.Length>1?args[1]:null);return 0;}
   if(mode=="--remove"){Remove();return 0;}
   if(mode=="--portable"){Stage(args.Length>1?args[1]:null);EnsureDriver();Process.Start(new ProcessStartInfo(Path.Combine(ProtectedDir,"DeskBox.Sensor.exe"),"--worker"){UseShellExecute=false,CreateNoWindow=true,WindowStyle=ProcessWindowStyle.Hidden});return 0;}
   if(mode=="--worker"){if(!Admin()||!String.Equals(Path.GetFullPath(AppDomain.CurrentDomain.BaseDirectory).TrimEnd(Path.DirectorySeparatorChar),ProtectedDir,StringComparison.OrdinalIgnoreCase))return 5;Serve();return 0;}
   if(mode=="--probe"){Probe();return 0;}
   return 2;
  }catch(Exception e){Console.Error.WriteLine(e.GetType().Name+": "+e.Message);if(args.Length>0&&(args[0]=="--install"||args[0]=="--remove"))System.Windows.Forms.MessageBox.Show(e.Message,"DeskBox CPU 温度",System.Windows.Forms.MessageBoxButtons.OK,System.Windows.Forms.MessageBoxIcon.Warning);return 1;}
 }
}
