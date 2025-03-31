/// Tests for the [INDIGO](https://github.com/indigo-astronomy/indigo/blob/master/indigo_docs/PROTOCOLS.md)
/// protocol extension of the [INDI](../doc/INDI.pdf) protocol.

const XML_INDIGO_REQ: &str = r#"
<getProperties client='My Client' version='2.0'/>
"#;

const XML_HANDSHAKE: &str = r#"
→ <getProperties version='1.7' client='My Client' switch='2.0'>
← <switchProtocol version='2.0'/>
"#;

const XML_ENABLE_BLOB: &str = r#"
→ <enableBLOB>URL</enableBLOB>
...
← <setBLOBVector device='CCD Simulator' name='CCD_IMAGE' state='Ok'>
    <oneBLOB name='IMAGE' url='http://*.'/>
  </setBLOBVector>
"#;

const X2J_GET_PROPERTIES: &str = r#"
→ <getProperties client='My Client' device='Server' name='LOAD' version='2.0'/>
→ { "getProperties": { "version": 512, "client": "My Client", "device": "Server", "name": "LOAD" } }
"#;

const X2J_DEF_TEXT_VECTOR: &str = r#"
← <defTextVector device='Server' name='LOAD' group='Main' label='Load driver' state='Idle' perm='rw'>
    <defText name='DRIVER' label='Load driver'></defText>
  </defTextVector>
← { "defTextVector": { "version": 512, "device": "Server", "name": "LOAD", "group": "Main", "label": "Load driver", "perm": "rw", "state": "Idle", "items": [  { "name": "DRIVER", "label": "Load driver", "value": "" } ] } }
"#;

const X2J_DEF_SWITCH_VECTOR: &str = r#"
← <defSwitchVector device='Server' name='RESTART' group='Main' label='Restart' rule='AnyOfMany' state='Idle' perm='rw'>
    <defSwitch name='RESTART' label='Restart server'>false</defSwitch>
  </defSwitchVector>
← { "defSwitchVector": { "version": 512, "device": "Server", "name": "RESTART", "group": "Main", "label": "Restart", "perm": "rw", "state": "Idle", "rule": "AnyOfMany", "hints": "order: 10; widget: button", "items": [  { "name": "RESTART", "label": "Restart server", "value": false } ] } }
"#;

const X2J_DEF_NUMBER_VECTOR: &str = r#"
← <defNumberVector device='CCD Imager Simulator' name='CCD_EXPOSURE' group='Camera' label='Start exposure' state='Idle' perm='rw'>
    <defNumber name='EXPOSURE' label='Start exposure' min='0' max='10000'step='1' format='%g' target='0'>0</defNumber>
  </defNumberVector>
← { "defNumberVector": { "version": 512, "device": "CCD Imager Simulator", "name": "CCD_EXPOSURE", "group": "Camera", "label": "Start exposure", "perm": "rw", "state": "Idle", "hints": "order: 10; target: show", "items": [  { "name": "EXPOSURE", "label": "Start exposure", "min": 0, "max": 10000, "step": 1, "format": "%g", "target": 0, "value": 0 } ] } }
"#;

const X2J_SET_SWITCH_VECTOR: &str = r#"
← <setSwitchVector device='CCD Imager Simulator' name='CONNECTION' state='Ok'>
    <oneSwitch name='CONNECTED'>On</oneSwitch>
    <oneSwitch name='DISCONNECTED'>Off</oneSwitch>
  </setSwitchVector>"
← { "setSwitchVector": { "device": "CCD Imager Simulator", "name": "CONNECTION", "state": "Ok", "items": [  { "name": "CONNECTED", "value": true }, { "name": "DISCONNECTED", "value": false } ] } }
"#;

const X2J_SET_BLOB_VECTOR: &str = r#"
← <setBLOBVector device='CCD Imager Simulator' name='CCD_IMAGE' state='Ok'>
	  <oneBLOB name='IMAGE'>/blob/0x10381d798.fits</oneSwitch>
  </setBLOBVector>
← { "setBLOBVector": { "device": "CCD Imager Simulator", "name": "CCD_IMAGE", "state": "Ok", "items": [  { "name": "IMAGE", "value": "/blob/0x10381d798.fits" } ] } }
"#;

const X2J_NEW_NUMBER_VECTOR: &str = r#"
→ <newNumberVector device='CCD Imager Simulator' name='CCD_EXPOSURE' token='FA0012'>
  	<oneNumber name='EXPOSURE'>1</defNumber>
  </newNumberVector>
→ {"newNumberVector":{"device":"CCD Imager Simulator","name":"CCD_EXPOSURE","token": "FA0012", "items":[{"name":"EXPOSURE","value":1}]}}
"#;

const X2J_DELETE_PROPERTY: &str = r#"
← <deleteProperty device='Mount IEQ (guider)'/>
← { "deleteProperty": { "device": "Mount IEQ (guider)" } }
"#;