<?xml version="1.0" encoding="UTF-8"?>
<tileset version="1.10" tiledversion="1.10.2" name="A3x3x3" tilewidth="30" tileheight="30" spacing="2" margin="2" tilecount="264" columns="12" objectalignment="bottom">
 <tileoffset x="-4" y="0"/>
 <grid orientation="isometric" width="28" height="14"/>
 <transformations hflip="1" vflip="0" rotate="0" preferuntransformed="0"/>
 <properties>
  <property name="Anchor::bottom_px" type="int" value="7"/>
 </properties>
 <image source="../img/spritesheetA_3x3x3.png" width="384" height="704"/>
 <tile id="0" type="Floor"/>
 <tile id="1" type="Floor"/>
 <tile id="2" type="Floor"/>
 <tile id="3" type="Floor"/>
 <tile id="4" type="Floor">
  <animation>
   <frame tileid="4" duration="1700"/>
   <frame tileid="6" duration="1740"/>
   <frame tileid="5" duration="1710"/>
   <frame tileid="7" duration="1840"/>
  </animation>
 </tile>
 <tile id="5" type="Floor">
  <animation>
   <frame tileid="5" duration="2840"/>
   <frame tileid="6" duration="2980"/>
   <frame tileid="7" duration="3310"/>
   <frame tileid="4" duration="3480"/>
  </animation>
 </tile>
 <tile id="6" type="Floor">
  <animation>
   <frame tileid="6" duration="4480"/>
   <frame tileid="4" duration="4380"/>
   <frame tileid="7" duration="4310"/>
   <frame tileid="6" duration="4140"/>
   <frame tileid="5" duration="4010"/>
  </animation>
 </tile>
 <tile id="7" type="Floor">
  <animation>
   <frame tileid="7" duration="3870"/>
   <frame tileid="5" duration="3760"/>
   <frame tileid="4" duration="3660"/>
   <frame tileid="6" duration="3620"/>
  </animation>
 </tile>
 <tile id="8" type="Floor"/>
 <tile id="9" type="Floor"/>
 <tile id="10" type="Floor"/>
 <tile id="11" type="Floor"/>
 <tile id="12" type="Floor"/>
 <tile id="13" type="Floor"/>
 <tile id="14" type="Floor"/>
 <tile id="15" type="Floor"/>
 <tile id="16" type="Floor"/>
 <tile id="17" type="Floor"/>
 <tile id="18" type="Floor"/>
 <tile id="19" type="Floor"/>
 <tile id="20" type="Floor"/>
 <tile id="21" type="Floor"/>
 <tile id="22" type="Floor"/>
 <tile id="23" type="Floor"/>
 <tile id="24" type="Wall"/>
 <tile id="36" type="Wall">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="9.33333" y="0" width="4.66667" height="14">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="37" type="Wall">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="9.33333" width="14" height="4.66667">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="38" type="Wall">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="9.33333" y="0" width="4.66667" height="14">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
   <object id="2" type="Shape" x="0" y="9.33333" width="9.33333" height="4.66667">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="39" type="DoorFrame">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="9.33333" width="14" height="4.66667">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="40" type="DoorFrame">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="9.33333" y="0" width="4.66667" height="14">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="41" type="Door">
  <properties>
   <property name="isOpen" type="bool" value="false"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="9.33333" y="0" width="4.66667" height="14">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="42" type="Door">
  <properties>
   <property name="is_open" type="bool" value="false"/>
   <property name="onactivate_transform_to_id" type="int" value="44"/>
  </properties>
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="9.33333" width="14" height="4.66667">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="43" type="Door">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="9.33333" width="14" height="4.66667">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="44" type="Door">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="9.33333" y="0" width="4.66667" height="14">
    <properties>
     <property name="collision" type="bool" value="false"/>
     <property name="interactive" type="bool" value="true"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="47">
  <objectgroup draworder="index" id="2">
   <object id="2" x="0" y="0" width="14" height="14"/>
  </objectgroup>
 </tile>
 <tile id="48" type="Decor">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="4.66667" width="9.33333" height="9.33333">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="49" type="Decor">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="0" y="0" width="9.33333" height="9.33333">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="50" type="Decor">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="4.66667" width="4.66667" height="4.66666">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="60" type="Decor">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="0" width="9.33333" height="14">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="61" type="Decor">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="4.66667" width="9.33333" height="9.33333">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="62" type="Decor"/>
 <tile id="63" type="Decor"/>
 <tile id="64" type="Decor"/>
 <tile id="65" type="Decor"/>
 <tile id="66" type="Decor"/>
 <tile id="67" type="Decor"/>
 <tile id="68" type="Decor"/>
 <tile id="69" type="Decor"/>
 <tile id="72" type="Decor">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="4.66667" width="4.66666" height="4.66666">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="73" type="Furniture">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="4.66667" width="4.66667" height="4.66667">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="74" type="Furniture">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="3.33333e-06" y="0" width="14" height="14">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="75" type="Furniture">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="1.16573e-15" width="9.33333" height="9.33333">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="84" type="Decor">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="4.66667" width="4.66667" height="4.66667">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="85" type="Decor">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="4.66667" width="4.66667" height="4.66667">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="86" type="Decor">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="4.66667" width="4.66667" height="4.66667">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="87" type="Decor">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="4.66667" width="4.66667" height="4.66667">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="88" type="Decor">
  <objectgroup draworder="index" id="2">
   <object id="1" type="Shape" x="4.66667" y="4.66667" width="4.66667" height="4.66667">
    <properties>
     <property name="collision" type="bool" value="true"/>
     <property name="interactive" type="bool" value="false"/>
    </properties>
   </object>
  </objectgroup>
 </tile>
 <tile id="96" type="Util">
  <properties>
   <property name="is_player_spawn" type="bool" value="true"/>
  </properties>
 </tile>
 <tile id="97" type="Util"/>
 <tile id="98" type="Util"/>
 <tile id="108" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Foyer"/>
  </properties>
 </tile>
 <tile id="109" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Living Room"/>
  </properties>
 </tile>
 <tile id="110" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Dining Room"/>
  </properties>
 </tile>
 <tile id="111" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Kitchen"/>
  </properties>
 </tile>
 <tile id="112" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Pantry"/>
  </properties>
 </tile>
 <tile id="113" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Mudroom"/>
  </properties>
 </tile>
 <tile id="114" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Family Room"/>
  </properties>
 </tile>
 <tile id="115" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Study"/>
  </properties>
 </tile>
 <tile id="116" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Library"/>
  </properties>
 </tile>
 <tile id="117" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Office"/>
  </properties>
 </tile>
 <tile id="118" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Home theater"/>
  </properties>
 </tile>
 <tile id="119" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Bathroom"/>
  </properties>
 </tile>
 <tile id="120" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Powder Room"/>
  </properties>
 </tile>
 <tile id="121" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Master Bedroom"/>
  </properties>
 </tile>
 <tile id="122" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Guest Bedroom"/>
  </properties>
 </tile>
 <tile id="123" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Children Bedroom"/>
  </properties>
 </tile>
 <tile id="124" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Laundry"/>
  </properties>
 </tile>
 <tile id="125" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Workshop"/>
  </properties>
 </tile>
 <tile id="126" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Garden"/>
  </properties>
 </tile>
 <tile id="127" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Basement"/>
  </properties>
 </tile>
 <tile id="128" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Attic"/>
  </properties>
 </tile>
 <tile id="129" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Utility Room"/>
  </properties>
 </tile>
 <tile id="130" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Storage Room"/>
  </properties>
 </tile>
 <tile id="131" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Closet"/>
  </properties>
 </tile>
 <tile id="132" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Mechanical Room"/>
  </properties>
 </tile>
 <tile id="133" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Hallway"/>
  </properties>
 </tile>
 <tile id="134" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Garage"/>
  </properties>
 </tile>
 <tile id="135" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Nursery"/>
  </properties>
 </tile>
 <tile id="136" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Teen Bedroom"/>
  </properties>
 </tile>
 <tile id="137" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Servant Bedroom"/>
  </properties>
 </tile>
 <tile id="138" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Half bath"/>
  </properties>
 </tile>
 <tile id="139" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Corridor"/>
  </properties>
 </tile>
 <tile id="140" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Water closet"/>
  </properties>
 </tile>
 <tile id="141" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Spare room"/>
  </properties>
 </tile>
 <tile id="142" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="Gallery"/>
  </properties>
 </tile>
 <tile id="143" type="RoomDef">
  <properties>
   <property name="RoomDef::display_name" value="En Suite"/>
  </properties>
 </tile>
 <wangsets>
  <wangset name="Unnamed Set" type="corner" tile="-1">
   <wangcolor name="stones" color="#ff0000" tile="-1" probability="1"/>
   <wangcolor name="grass" color="#00ff00" tile="-1" probability="1"/>
   <wangcolor name="asphalt" color="#0000ff" tile="-1" probability="1"/>
   <wangtile tileid="0" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="1" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="2" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="3" wangid="0,1,0,1,0,1,0,1"/>
   <wangtile tileid="4" wangid="0,2,0,2,0,2,0,2"/>
   <wangtile tileid="5" wangid="0,2,0,2,0,2,0,2"/>
   <wangtile tileid="6" wangid="0,2,0,2,0,2,0,2"/>
   <wangtile tileid="7" wangid="0,2,0,2,0,2,0,2"/>
   <wangtile tileid="8" wangid="0,3,0,3,0,3,0,3"/>
   <wangtile tileid="9" wangid="0,3,0,3,0,3,0,3"/>
   <wangtile tileid="10" wangid="0,3,0,3,0,3,0,3"/>
   <wangtile tileid="11" wangid="0,3,0,3,0,3,0,3"/>
  </wangset>
  <wangset name="Unnamed Set" type="edge" tile="-1">
   <wangcolor name="" color="#ff0000" tile="-1" probability="1"/>
  </wangset>
 </wangsets>
</tileset>
