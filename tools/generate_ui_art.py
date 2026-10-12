from PIL import Image, ImageDraw
from pathlib import Path
out=Path('assets/ui/icons')
S=4
for name in ['attack','guard','spell','combo','item','flee']:
    im=Image.new('RGBA',(64*S,64*S)); d=ImageDraw.Draw(im)
    gold=(225,195,135,255); pale=(242,232,204,255)
    def line(p,c=gold,w=3): d.line([(x*S,y*S) for x,y in p],fill=c,width=w*S,joint='curve')
    def poly(p,c=pale): d.polygon([(x*S,y*S) for x,y in p],fill=c)
    def ellipse(b,c=gold,w=2): d.ellipse(tuple(v*S for v in b),outline=c,width=w*S)
    if name=='attack':
        poly([(27,37),(43,12),(51,9),(49,18),(33,42)]); line([(21,34),(39,46)]); line([(28,42),(20,54)],w=5); ellipse((15,50,23,58))
    elif name=='guard':
        line([(32,9),(50,17),(48,37),(42,47),(32,55),(22,47),(16,37),(14,17),(32,9)],w=3); line([(32,19),(32,43)]); line([(22,30),(42,30)])
    elif name=='spell':
        ellipse((11,11,53,53)); ellipse((17,17,47,47),w=1); poly([(32,19),(38,32),(32,45),(26,32)]); line([(6,32),(15,32)]); line([(49,32),(58,32)]); line([(32,6),(32,15)])
    elif name=='combo':
        poly([(36,8),(20,34),(31,34),(25,56),(46,27),(34,27)]); line([(13,19),(8,27)],w=2); line([(51,40),(56,32)],w=2)
    elif name=='item':
        line([(25,11),(39,11)],w=4); line([(26,15),(26,25),(16,36),(16,49),(22,55),(42,55),(48,49),(48,36),(38,25),(38,15)],w=3); line([(20,40),(44,40)],w=2); line([(32,33),(32,49)],pale); line([(25,41),(39,41)],pale)
    else:
        line([(28,10),(15,10),(15,54),(28,54)]); line([(25,32),(55,32)],pale); line([(45,21),(56,32),(45,43)],pale); line([(20,16),(28,20),(28,47),(20,50)],w=2)
    im.resize((64,64),Image.Resampling.LANCZOS).save(out/f'{name}.png')
# Code-owned soft contact shadow, independent of character artwork.
im=Image.new('RGBA',(128,64)); pix=im.load()
for y in range(64):
    for x in range(128):
        r=((x-63.5)/63.5)**2+((y-31.5)/31.5)**2
        pix[x,y]=(5,9,12,int(190*max(0,1-r)**2))
im.save('assets/effects/contact_shadow.png')
import math
# Soft, engraved spell rings: colored in-engine, transparency does the shading.
for name,inner in [('spell-ring',.86),('spell-ring-wide',.70)]:
    im=Image.new('RGBA',(256,256)); pix=im.load()
    for y in range(256):
        for x in range(256):
            dx=(x-127.5)/127.5; dy=(y-127.5)/127.5; r=math.hypot(dx,dy); a=math.atan2(dy,dx)
            core=math.exp(-((r-(1+inner)/2)/((1-inner)*.27))**2)
            rim=math.exp(-((r-.98)/.008)**2)*.45
            etch=(.6+.4*math.sin(a*24+.8))*math.exp(-((r-inner)/.01)**2)
            alpha=min(1,core*.75+rim+etch*.5)*max(0,min(1,(1-r)/.012))
            pix[x,y]=(255,255,255,int(alpha*255))
    im.save(f'assets/effects/{name}.png')
im=Image.new('RGBA',(184,256));d=ImageDraw.Draw(im)
d.polygon([(92,3),(19,245),(165,245)],fill=(170,205,223,210))
d.polygon([(92,3),(92,240),(19,245)],fill=(247,252,255,235))
d.polygon([(92,3),(165,245),(112,214)],fill=(91,143,184,130))
d.line([(92,8),(92,230),(27,240)],fill=(255,255,255,230),width=3)
d.line([(92,144),(65,174),(87,213)],fill=(225,245,255,160),width=2)
im.save('assets/effects/spell-shard.png')
