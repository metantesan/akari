// Akari liquid glass. Each glass primitive samples a snapshot of the layers
// beneath it. Never sample the active render attachment.
struct Globals { viewport: vec4<f32> }
@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var linear_sampler: sampler;
@group(0) @binding(3) var backdrop: texture_2d<f32>;
struct Input {
 @location(0) rect: vec4<f32>, @location(1) clip: vec4<f32>,
 @location(2) color: vec4<f32>, @location(3) uv: vec4<f32>,
 @location(4) params: vec4<f32>, @location(5) optical: vec4<f32>,
}
struct Varying {
 @builtin(position) position: vec4<f32>, @location(0) local: vec2<f32>,
 @location(1) @interpolate(flat) rect: vec4<f32>,
 @location(2) @interpolate(flat) clip: vec4<f32>,
 @location(3) @interpolate(flat) color: vec4<f32>,
 @location(4) uv: vec2<f32>,
 @location(5) @interpolate(flat) params: vec4<f32>,
 @location(6) @interpolate(flat) optical: vec4<f32>,
}
@vertex fn vs(i:Input,@builtin(vertex_index) vertex:u32)->Varying {
 let corners=array<vec2<f32>,6>(vec2(0.,0.),vec2(1.,0.),vec2(0.,1.),vec2(0.,1.),vec2(1.,0.),vec2(1.,1.));
 let p=corners[vertex]; let xy=i.rect.xy+p*i.rect.zw;
 var o:Varying; o.position=vec4(xy/globals.viewport.xy*vec2(2.,-2.)+vec2(-1.,1.),0.,1.);
 o.local=p*i.rect.zw; o.rect=i.rect; o.clip=i.clip; o.color=i.color;
 o.uv=i.uv.xy+p*i.uv.zw; o.params=i.params; o.optical=i.optical;return o;
}
fn rounded(p:vec2<f32>,size:vec2<f32>,radius:f32)->f32 {
 let r=clamp(radius,0.,min(size.x,size.y)*0.5);
 let q=abs(p-size*0.5)-size*0.5+vec2(r);
 return length(max(q,vec2(0.)))+min(max(q.x,q.y),0.)-r;
}
@fragment fn fs(i:Varying)->@location(0) vec4<f32> {
 let xy=i.position.xy;
 if any(xy<i.clip.xy)||any(xy>=i.clip.xy+i.clip.zw){discard;}
 let mode=i.params.z;
 if mode>2.5 && mode<3.5 {
  let point=i.local/i.rect.zw*24.;
  let a=i.optical.xy; let b=i.optical.zw;
  let ab=b-a;
  let t=clamp(dot(point-a,ab)/max(dot(ab,ab),0.00001),0.,1.);
  let distance=length(point-(a+t*ab));
  let aa=max(fwidth(distance),0.025);
  let coverage=1.-smoothstep(i.params.x-aa,i.params.x+aa,distance);
  if coverage<=0. { discard; }
  return vec4(i.color.rgb,i.color.a*coverage);
 }
 if mode>0.5 && mode<1.5 {return textureSample(atlas,linear_sampler,i.uv)*i.color;}
 let distance=rounded(i.local,i.rect.zw,i.params.x);
 let aa=max(fwidth(distance),0.65);let coverage=1.-smoothstep(-aa,aa,distance);
 if coverage<=0.{discard;}
 if mode>1.5 { // Liquid glass: curved edge normals bend the captured backdrop.
  let normal=normalize(vec2(dpdx(distance),dpdy(distance))+vec2(0.00001));
  let edge=exp(-abs(distance)/max(i.params.x*0.45,1.));
  let refracted=xy+normal*i.optical.y*edge;
  let blur=max(i.optical.x,0.);
  var sum=vec3(0.);var weights=0.;
  // A normalized 5x5 Gaussian kernel, with independent blur and refraction.
  for(var y=-2;y<=2;y++){for(var x=-2;x<=2;x++){
   let offset=vec2(f32(x),f32(y));let weight=exp(-dot(offset,offset)*0.5);
   let uv=clamp((refracted+offset*blur*0.5)/globals.viewport.xy,vec2(0.),vec2(1.));
   sum+=textureSampleLevel(backdrop,linear_sampler,uv,0.).rgb*weight;weights+=weight;
  }}
  let base=mix(sum/weights,i.color.rgb,i.color.a);
  let rim=(1.-smoothstep(i.optical.z,i.optical.z+1.4,abs(distance)));
  let light=0.45+0.55*max(dot(normal,normalize(vec2(-0.65,-0.76))),0.);
  let sheen=pow(max(0.,1.-i.local.y/max(i.rect.w,1.)),8.)*0.035;
  return vec4(base+vec3(rim*light*0.42+sheen),coverage);
 }
 var alpha=coverage*i.color.a;
 if i.params.y>0. {alpha*=smoothstep(-i.params.y-aa,-i.params.y+aa,distance);}
 return vec4(i.color.rgb,alpha);
}
