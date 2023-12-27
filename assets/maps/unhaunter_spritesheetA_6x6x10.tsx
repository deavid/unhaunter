<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.2" name="A6x6x10" tilewidth="48" tileheight="64" tilecount="64" columns="8" fillmode="preserve-aspect-fit">
 <tileoffset x="-12" y="12"/>
 <grid orientation="isometric" width="48" height="24"/>
 <image source="../img/spritesheetA_6x6x10.png" width="384" height="512"/>
 <tile id="0">
  <objectgroup draworder="index" id="2">
   <object id="1" x="28.4444" y="21.3333" width="21.3333" height="35.5556"/>
   <object id="2" x="35.5556" y="35.5556" width="14.2222" height="7.11111"/>
   <object id="3" x="35.5556" y="21.3333"/>
   <object id="4" x="16" y="28" width="5" height="7"/>
  </objectgroup>
 </tile>
 <tile id="2" type="Light">
  <properties>
   <property name="light_power" type="float" value="1e-05"/>
   <property name="onactivate_transform_to_id" type="int" value="3"/>
  </properties>
 </tile>
 <tile id="3" type="Light">
  <properties>
   <property name="light_power" type="float" value="100"/>
   <property name="onactivate_transform_to_id" type="int" value="2"/>
  </properties>
 </tile>
 <tile id="4">
  <objectgroup draworder="index" id="3">
   <object id="5" x="42.6667" y="49.7778"/>
   <object id="9" x="0" y="42.6667"/>
   <object id="12" x="35.5556" y="35.5556" width="14.2222" height="14.2222"/>
   <object id="13" x="42.6667" y="42.6667"/>
   <object id="14" x="42.6667" y="49.7778"/>
   <object id="15" x="49.7778" y="56.8889"/>
   <object id="16" x="64" y="42.6667"/>
  </objectgroup>
 </tile>
 <tile id="8" type="WallObject"/>
 <tile id="11">
  <objectgroup draworder="index" id="2">
   <object id="1" x="18" y="24" width="6" height="10"/>
   <object id="2" x="21.3333" y="28.4444" width="7.11111" height="7.11111"/>
   <object id="3" x="42.6667" y="28.4444"/>
  </objectgroup>
 </tile>
 <tile id="18">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="0" width="12" height="23"/>
   <object id="2" x="8" y="2.66667"/>
  </objectgroup>
 </tile>
</tileset>
