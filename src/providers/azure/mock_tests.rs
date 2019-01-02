use mockito::{self, Matcher};
use providers::{azure, MetadataProvider};

#[test]
fn test_boot_checkin() {
    let fab_version = "/?comp=versions";
    let ver_body = r#"<?xml version="1.0" encoding="utf-8"?>
<Versions>
  <Preferred>
    <Version>2015-04-05</Version>
  </Preferred>
  <Supported>
    <Version>2015-04-05</Version>
    <Version>2012-11-30</Version>
    <Version>2012-09-15</Version>
    <Version>2012-05-15</Version>
    <Version>2011-12-31</Version>
    <Version>2011-10-15</Version>
    <Version>2011-08-31</Version>
    <Version>2011-04-07</Version>
    <Version>2010-12-15</Version>
    <Version>2010-28-10</Version>
  </Supported>
</Versions>"#;
    let m_version = mockito::mock("GET", fab_version)
        .with_body(ver_body)
        .with_status(200)
        .create();

    let fab_goalstate = "/machine/?comp=goalstate";
    let gs_body = r#"<?xml version="1.0" encoding="utf-8"?>
<GoalState xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="goalstate10.xsd">
  <Version>2012-11-30</Version>
  <Incarnation>1</Incarnation>
  <Machine>
    <ExpectedState>Started</ExpectedState>
    <StopRolesDeadlineHint>300000</StopRolesDeadlineHint>
    <LBProbePorts>
      <Port>16001</Port>
    </LBProbePorts>
    <ExpectHealthReport>FALSE</ExpectHealthReport>
  </Machine>
  <Container>
    <ContainerId>a511aa6d-29e7-4f53-8788-55655dfe848f</ContainerId>
    <RoleInstanceList>
      <RoleInstance>
        <InstanceId>f6cd1d7ef1644557b9059345e5ba890c.lars-test-1</InstanceId>
        <State>Started</State>
        <Configuration>
          <HostingEnvironmentConfig>http://100.115.176.3:80/machine/a511aa6d-29e7-4f53-8788-55655dfe848f/f6cd1d7ef1644557b9059345e5ba890c.lars%2Dtest%2D1?comp=config&amp;type=hostingEnvironmentConfig&amp;incarnation=1</HostingEnvironmentConfig>
          <SharedConfig>http://100.115.176.3:80/machine/a511aa6d-29e7-4f53-8788-55655dfe848f/f6cd1d7ef1644557b9059345e5ba890c.lars%2Dtest%2D1?comp=config&amp;type=sharedConfig&amp;incarnation=1</SharedConfig>
          <ExtensionsConfig>http://100.115.176.3:80/machine/a511aa6d-29e7-4f53-8788-55655dfe848f/f6cd1d7ef1644557b9059345e5ba890c.lars%2Dtest%2D1?comp=config&amp;type=extensionsConfig&amp;incarnation=1</ExtensionsConfig>
          <FullConfig>http://100.115.176.3:80/machine/a511aa6d-29e7-4f53-8788-55655dfe848f/f6cd1d7ef1644557b9059345e5ba890c.lars%2Dtest%2D1?comp=config&amp;type=fullConfig&amp;incarnation=1</FullConfig>
          <Certificates>http://100.115.176.3:80/machine/a511aa6d-29e7-4f53-8788-55655dfe848f/f6cd1d7ef1644557b9059345e5ba890c.lars%2Dtest%2D1?comp=certificates&amp;incarnation=1</Certificates>
          <ConfigName>f6cd1d7ef1644557b9059345e5ba890c.0.f6cd1d7ef1644557b9059345e5ba890c.0.lars-test-1.1.xml</ConfigName>
        </Configuration>
      </RoleInstance>
    </RoleInstanceList>
  </Container>
</GoalState>
"#;
    let m_goalstate = mockito::mock("GET", fab_goalstate)
        .with_body(gs_body)
        .with_status(200)
        .create();

    let fab_health = "/machine/?comp=health";
    let m_health = mockito::mock("POST", fab_health)
        .match_header("content-type", Matcher::Regex("text/xml".to_string()))
        .match_header("x-ms-version", Matcher::Regex("2012-11-30".to_string()))
        .match_body(Matcher::Regex("<State>Ready</State>".to_string()))
        .with_status(200)
        .create();

    let provider = azure::Azure::try_new();
    let r = provider.unwrap().boot_checkin();

    m_version.assert();
    m_goalstate.assert();
    m_health.assert();
    r.unwrap();
}
