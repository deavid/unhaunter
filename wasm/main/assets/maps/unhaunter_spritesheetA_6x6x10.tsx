<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.2" name="A6x6x10" tilewidth="48" tileheight="64" tilecount="64" columns="8" fillmode="preserve-aspect-fit">
 <tileoffset x="-12" y="12"/>
 <grid orientation="isometric" width="48" height="24"/>
 <properties>
  <property name="Anchor::bottom_px" type="int" value="19"/>
 </properties>
 <image source="../img/spritesheetA_6x6x10.png" width="384" height="512"/>
 <tile id="0" type="Switch">
  <properties>
   <property name="sprite:orientation" value="YAxis"/>
   <property name="sprite:state" value="Off"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="3" x="35.5556" y="21.3333"/>
   <object id="4" type="Shape" x="0" y="0" width="4" height="12">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="1" type="Switch">
  <properties>
   <property name="sprite:orientation" value="YAxis"/>
   <property name="sprite:state" value="On"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="2" type="Shape" x="0" y="0" width="4" height="12">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="2" type="WallLamp">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="Off"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="4" height="12">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="3" type="WallLamp">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="On"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="4" height="12">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="4" type="FloorLamp">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="Off"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="3">
   <object id="5" x="42.6667" y="49.7778"/>
   <object id="9" x="0" y="42.6667"/>
   <object id="13" x="42.6667" y="42.6667"/>
   <object id="14" x="42.6667" y="49.7778"/>
   <object id="15" x="49.7778" y="56.8889"/>
   <object id="16" x="64" y="42.6667"/>
   <object id="18" type="Shape" x="0" y="0" width="8" height="8">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="5" type="FloorLamp">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="On"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="2" type="Shape" x="0" y="0" width="8" height="8">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="6" type="TableLamp">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="Off"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="2" type="Shape" x="0" y="-4" width="12" height="16">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="7" type="TableLamp">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="On"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="2" type="Shape" x="0" y="-4" width="12" height="16">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="8" type="WallDecor">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Mirror"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="4" width="4" height="12">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="9" type="WallDecor">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Clock"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="4" height="12">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="10" type="CeilingLight">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="On"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" x="4" y="4" width="4" height="4">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="11" type="RoomSwitch">
  <properties>
   <property name="sprite:orientation" value="YAxis"/>
   <property name="sprite:state" value="Off"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="3" x="42.6667" y="28.4444"/>
   <object id="4" type="Shape" x="0" y="0" width="4" height="12">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="12" type="RoomSwitch">
  <properties>
   <property name="sprite:orientation" value="YAxis"/>
   <property name="sprite:state" value="On"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="4" height="12">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="13" type="Breaker">
  <properties>
   <property name="sprite:orientation" value="YAxis"/>
   <property name="sprite:state" value="Off"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="8" height="12">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="14" type="Breaker">
  <properties>
   <property name="sprite:orientation" value="YAxis"/>
   <property name="sprite:state" value="On"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="-3.33333e-06" y="3.33333e-06" width="8" height="12">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="15" type="StreetLight">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="On"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
 </tile>
 <tile id="16" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="GreenCouch"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="-4" width="16" height="28">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="17" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="RedSofa"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="16" height="24">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="18" type="Appliance">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="TV"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="2" x="8" y="2.66667"/>
   <object id="3" type="Shape" x="0" y="0" width="12" height="24">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="19" type="WallDecor">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Bookshelf"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="4" height="24">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="20" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Bed"/>
    <property name="object:pickable" type="bool" value="false" />
    <property name="object:movable" type="bool" value="false" />
    <property name="object:hidingspot" type="bool" value="true" />
    <property name="object:weight" type="float" value="30.0" />
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="-6.66134e-16" y="-5.33333" width="20" height="29.3333">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="21" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Bathtub"/>
    <property name="object:pickable" type="bool" value="false" />
    <property name="object:movable" type="bool" value="false" />
    <property name="object:hidingspot" type="bool" value="true" />
    <property name="object:weight" type="float" value="30.0" />
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="16" height="24">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="22" type="Appliance">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="WashingMachine"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="12" height="16">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="23" type="Appliance">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Fridge"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.44089e-16" y="-3.10862e-15" width="20" height="16">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="24" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="GreenTable"/>
    <property name="object:pickable" type="bool" value="false" />
    <property name="object:movable" type="bool" value="false" />
    <property name="object:hidingspot" type="bool" value="true" />
    <property name="object:weight" type="float" value="30.0" />
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4" y="-4" width="12" height="28">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="25" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="OakTable"/>
    <property name="object:pickable" type="bool" value="false" />
    <property name="object:movable" type="bool" value="false" />
    <property name="object:hidingspot" type="bool" value="true" />
    <property name="object:weight" type="float" value="30.0" />
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4" y="-4" width="12" height="28">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="26" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="WoodTable"/>
    <property name="object:pickable" type="bool" value="false" />
    <property name="object:movable" type="bool" value="false" />
    <property name="object:hidingspot" type="bool" value="true" />
    <property name="object:weight" type="float" value="30.0" />
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4" y="-4" width="12" height="28">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="27" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Desk"/>
    <property name="object:pickable" type="bool" value="false" />
    <property name="object:movable" type="bool" value="false" />
    <property name="object:hidingspot" type="bool" value="true" />
    <property name="object:weight" type="float" value="30.0" />
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4" y="-4" width="12" height="28">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="28" type="Appliance">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Stove"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="-8" width="12" height="32">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="29" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Sink"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="-8" width="12" height="32">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="32" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="EmptyBookshelf"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="12" height="20">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="33" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Bookshelf"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="12" height="20">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="40" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Drawer"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="12" height="24">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="41" type="Furniture">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Wardrobe"/>
    <property name="object:pickable" type="bool" value="false" />
    <property name="object:movable" type="bool" value="false" />
    <property name="object:hidingspot" type="bool" value="true" />
    <property name="object:weight" type="float" value="30.0" />
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="12" height="24">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="48" type="Window">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="Window"/>
  </properties>
 </tile>
 <tile id="49" type="Window">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="RedCurtainWindow"/>
  </properties>
 </tile>
 <tile id="50" type="Window">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="GreenCurtainWindow"/>
  </properties>
 </tile>
 <tile id="56" type="WallDecor">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="BlankPicture"/>
  </properties>
 </tile>
 <tile id="57" type="WallDecor">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="GreenPicture"/>
  </properties>
 </tile>
 <tile id="58" type="WallDecor">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="None"/>
   <property name="sprite:variant" value="RedPicture"/>
  </properties>
 </tile>
 <tile id="62" type="CeilingLight">
  <properties>
   <property name="sprite:orientation" value="None"/>
   <property name="sprite:state" value="Off"/>
   <property name="sprite:variant" value="Base"/>
  </properties>
 </tile>
 <tile id="63">
  <objectgroup draworder="index" id="2">
   <object id="1" x="0" y="24"/>
   <object id="3" x="0" y="0" width="12" height="12"/>
   <object id="4" x="5.33333" y="10.6667"/>
  </objectgroup>
 </tile>
</tileset>
 