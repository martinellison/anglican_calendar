//! fixes for tags
use std::collections::HashMap;
/** tags that are remapped to other tags. Reused when two different
 * names are used for the same event in different calendars.  */
pub fn tag_fixes() -> HashMap<String, String> {
    let mut tf: HashMap<String, String> = HashMap::new();
    tf.insert("óscar romero".to_string(), "oscar romero".to_string());
    tf.insert("aelred".to_string(), "aelred of hexham".to_string()); //??
    tf.insert("aidan of lindisfarne".to_string(), "aidan".to_string()); //??
    tf.insert("all saints".to_string(), "all saints' day".to_string());
    tf.insert("ambrose of milan".to_string(), "ambrose".to_string());
    tf.insert("andrew".to_string(), "andrew the apostle".to_string());
    tf.insert("ansgar".to_string(), "anskar".to_string()); //??
    tf.insert("barnabas".to_string(), "barnabas the apostle".to_string());
    tf.insert(
        "bartholomew".to_string(),
        "bartholomew the apostle".to_string(),
    );
    tf.insert("bede of jarorw".to_string(), "bede".to_string());
    tf.insert("christmas".to_string(), "christmas day".to_string());
    tf.insert(
        "the nativity of jesus christ".to_string(),
        "christmas day".to_string(),
    );
    tf.insert(
        "commemoration of all faithful departed".to_string(),
        "commemoration of the faithful departed (all souls' day)".to_string(),
    );
    tf.insert(
        "cuthbert of lindisfarne".to_string(),
        "cuthbert".to_string(),
    );
    tf.insert("cyprian of carthage".to_string(), "cyprian".to_string());
    tf.insert("hilda of whitby".to_string(), "hilda".to_string());
    tf.insert("hildegard of bingen".to_string(), "hildegard".to_string());
    tf.insert("irenaeus".to_string(), "irenæus".to_string());
    tf.insert("julian".to_string(), "julian of norwich".to_string());
    tf.insert("justin martyr".to_string(), "justin".to_string());
    tf.insert("lawrence".to_string(), "laurence".to_string());
    tf.insert("leo i".to_string(), "leo the great".to_string());
    tf.insert("leo of rome".to_string(), "leo the great".to_string());
    tf.insert("luke".to_string(), "luke the evangelist".to_string());
    tf.insert("mark".to_string(), "mark the evangelist".to_string());
    tf.insert("martin of tours".to_string(), "martin".to_string());
    tf.insert("monnica of hippo".to_string(), "monica".to_string());
    tf.insert(
        "the presentation of christ in the temple".to_string(),
        "the presentation of christ in the temple (candlemas)".to_string(),
    );
    tf.insert(
        "the presentation of our lord jesus christ in the temple".to_string(),
        "the presentation of christ in the temple (candlemas)".to_string(),
    );
    tf.insert(
        "the transfiguration of our lord jesus christ".to_string(),
        "the transfiguration of our lord".to_string(),
    );
    tf.insert("thomas".to_string(), "thomas the apostle".to_string());
    tf.insert(
        "saint thomas the apostle".to_string(),
        "thomas the apostle".to_string(),
    );
    tf.insert("saint andrew".to_string(), "andrew the apostle".to_string());
    tf.insert(
        "saint andrew the apostle".to_string(),
        "andrew the apostle".to_string(),
    );
    tf.insert(
        "saint mark the evangelist".to_string(),
        "mark the evangelist".to_string(),
    );
    tf.insert(
        "saint luke the evangelist".to_string(),
        "luke the evangelist".to_string(),
    );
    tf.insert(
        "transfiguration of jesus".to_string(),
        "the transfiguration of our lord".to_string(),
    );
    tf.insert("bede".to_string(), "the venerable bede".to_string());
    tf.insert(
        "the visitation of the blessed virgin mary".to_string(),
        "the visit of the blessed virgin mary to elizabeth".to_string(),
    );
    tf.insert(
        "the visitation of the blessed virgin mary to elizabeth".to_string(),
        "the visit of the blessed virgin mary to elizabet".to_string(),
    );
    tf.insert(
        "saint barnabas the apostle".to_string(),
        "barnabas the apostle".to_string(),
    ); //??
    tf.insert(
        "saint bartholomew the apostle".to_string(),
        "bartholomew the apostle".to_string(),
    );
    tf.insert("saint dominic".to_string(), "dominic".to_string());
    tf.insert("saint john".to_string(), "john".to_string());
    tf.insert("saint joseph".to_string(), "joseph".to_string());
    tf.insert("saint jude".to_string(), "jude".to_string());
    tf.insert(
        "saint mary magdalene".to_string(),
        "mary magdalene".to_string(),
    ); //??
    tf.insert(
        "saint mary the virgin".to_string(),
        "mary the virgin".to_string(),
    ); //??
    tf.insert("saint matthew".to_string(), "matthew".to_string());
    tf.insert("matthew the evangelist".to_string(), "matthew".to_string());
    tf.insert(
        "saint matthias the apostle".to_string(),
        "matthias the apostle".to_string(),
    );
    tf.insert("matthias".to_string(), "matthias the apostle".to_string());
    tf.insert(
        "saint michael and all angels".to_string(),
        "michael and all angels".to_string(),
    );
    tf.insert("michael".to_string(), "michael and all angels".to_string());
    tf.insert("saint paul the apostle".to_string(), "paul".to_string());
    tf.insert(
        "saint peter and saint paul".to_string(),
        "peter and paul".to_string(),
    ); //??
    tf.insert("saint stephen".to_string(), "stephen".to_string()); //??

    tf.insert(
        "the annunciation of our lord jesus christ to the blessed virgin mary".to_string(),
        "the annunciation of our lord to the blessed virgin mary".to_string(),
    );
    tf.insert(
        "the annunciation to the blessed virgin mary".to_string(),
        "the annunciation of our lord to the blessed virgin mary".to_string(),
    );
    tf.insert(
        "annunciation".to_string(),
        "the annunciation of our lord to the blessed virgin mary".to_string(),
    );
    tf.insert(
        "the epiphany of our lord jesus christ".to_string(),
        "the epiphany".to_string(),
    );
    tf.insert(
        "the visit of the blessed virgin mary to elizabet".to_string(),
        "the visit of the blessed virgin mary to elizabeth".to_string(),
    );
    tf.insert("jeanne d’arc".to_string(), "joan of arc".to_string());
    tf.insert("john the apostle".to_string(), "john".to_string()); //??
    tf.insert("john wycliffe".to_string(), "john wyclif".to_string());
    tf.insert("nicholas of myra".to_string(), "nicholas".to_string());
    tf.insert("remegius".to_string(), "remigius".to_string());
    tf.insert(
        "scholastica of nursia".to_string(),
        "scholastica".to_string(),
    );
    tf.insert(
        "teresa of ávila".to_string(),
        "teresa of avila".to_string(),
    );
    tf.insert("willibrord".to_string(), "willibrord of york".to_string());
    tf.insert("oswald of northumbria".to_string(), "oswald".to_string());
    tf.insert("ninian of galloway".to_string(), "ninian".to_string());
    tf.insert("joseph".to_string(), "joseph of nazareth".to_string());
    tf.insert("ignatius of antioch".to_string(), "ignatius".to_string());
    tf.insert("holy cross".to_string(), "holy cross day".to_string());
    tf.insert("hilary of poitiers".to_string(), "hilary".to_string());
    tf.insert(
        "all souls' day".to_string(),
        "commemoration of the faithful departed (all souls' day)".to_string(),
    );
    // tf.insert("".to_string(), "".to_string());
    // tf.insert("".to_string(), "".to_string());
    tf
}
