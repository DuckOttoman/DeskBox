(()=>{
 const valid=n=>typeof n==='number'&&Number.isFinite(n)&&n>=0;
 const text=s=>`${valid(s.vramUsed)?(s.vramUsed/1024**3).toFixed(1):'—'} / ${valid(s.vramTotal)?(s.vramTotal/1024**3).toFixed(1):'—'} GB`;
 const ratio=s=>valid(s.vramUsed)&&valid(s.vramTotal)&&s.vramTotal>0?Math.min(1,s.vramUsed/s.vramTotal):0;
 if(typeof module!=='undefined')module.exports={text,ratio};if(typeof window!=='undefined')window.DeskBoxVram={text,ratio};
})();
