use crate::providers::{microsoft::azure, MetadataProvider};
use crate::retry;
use mockito::{self, Matcher};

/// Response body for goalstate (with certificates endpoint).
static GOALSTATE_BODY: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<GoalState xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="goalstate10.xsd">
  <Version>2012-11-30</Version>
  <Incarnation>42</Incarnation>
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

/// Response body for goalstate (without certificates endpoint).
static GOALSTATE_BODY_NO_CERTS: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<GoalState xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="goalstate10.xsd">
  <Version>2012-11-30</Version>
  <Incarnation>42</Incarnation>
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
          <ConfigName>f6cd1d7ef1644557b9059345e5ba890c.0.f6cd1d7ef1644557b9059345e5ba890c.0.lars-test-1.1.xml</ConfigName>
        </Configuration>
      </RoleInstance>
    </RoleInstanceList>
  </Container>
</GoalState>
"#;

fn mock_fab_version(server: &mut mockito::Server) -> mockito::Mock {
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

    server
        .mock("GET", fab_version)
        .with_body(ver_body)
        .with_status(200)
        .create()
}

fn mock_goalstate(server: &mut mockito::Server, with_certificates: bool) -> mockito::Mock {
    let fab_goalstate = "/machine/?comp=goalstate";

    let gs_body = if with_certificates {
        GOALSTATE_BODY
    } else {
        GOALSTATE_BODY_NO_CERTS
    };

    server
        .mock("GET", fab_goalstate)
        .with_body(gs_body)
        .with_status(200)
        .create()
}

#[test]
fn test_boot_checkin() {
    let mut server = mockito::Server::new();
    let m_version = mock_fab_version(&mut server);
    let m_goalstate = mock_goalstate(&mut server, true);

    let fab_health = "/machine/?comp=health";
    let m_health = server
        .mock("POST", fab_health)
        .match_header("content-type", Matcher::Regex("text/xml".to_string()))
        .match_header("x-ms-version", Matcher::Regex("2012-11-30".to_string()))
        .match_body(Matcher::Regex("<State>Ready</State>".to_string()))
        .match_body(Matcher::Regex(
            "<GoalStateIncarnation>42</GoalStateIncarnation>".to_string(),
        ))
        .with_status(200)
        .create();

    let client = retry::Client::try_new()
        .unwrap()
        .mock_base_url(server.url());
    let provider = azure::Azure::with_client(Some(client)).unwrap();
    let r = provider.boot_checkin();

    m_version.assert();
    m_goalstate.assert();
    m_health.assert();
    r.unwrap();

    server.reset();

    // Check error logic, but fail fast without re-trying.
    let client = retry::Client::try_new()
        .unwrap()
        .max_retries(0)
        .mock_base_url(server.url());
    azure::Azure::with_client(Some(client)).unwrap_err();
}

#[test]
fn test_hostname() {
    let mut server = mockito::Server::new();
    let m_version = mock_fab_version(&mut server);

    let testname = "testname";
    let endpoint = "/metadata/instance/compute/name?api-version=2017-08-01&format=text";
    let m_hostname = server
        .mock("GET", endpoint)
        .match_header("Metadata", "true")
        .with_body(testname)
        .with_status(200)
        .create();

    let client = retry::Client::try_new()
        .unwrap()
        .mock_base_url(server.url());
    let provider = azure::Azure::with_client(Some(client)).unwrap();
    let r = provider.hostname().unwrap();

    m_version.assert();

    m_hostname.assert();
    let hostname = r.unwrap();
    assert_eq!(hostname, testname);

    server.reset();

    // Check error logic, but fail fast without re-trying.
    let client = retry::Client::try_new()
        .unwrap()
        .max_retries(0)
        .mock_base_url(server.url());
    azure::Azure::with_client(Some(client)).unwrap_err();
}

#[test]
fn test_vmsize() {
    let mut server = mockito::Server::new();
    let m_version = mock_fab_version(&mut server);

    let testvmsize = "testvmsize";
    let endpoint = "/metadata/instance/compute/vmSize?api-version=2017-08-01&format=text";
    let m_vmsize = server
        .mock("GET", endpoint)
        .match_header("Metadata", "true")
        .with_body(testvmsize)
        .with_status(200)
        .create();

    let client = retry::Client::try_new()
        .unwrap()
        .mock_base_url(server.url());
    let provider = azure::Azure::with_client(Some(client)).unwrap();
    let attributes = provider.attributes().unwrap();
    let r = attributes.get("AZURE_VMSIZE");

    m_version.assert();

    m_vmsize.assert();
    let vmsize = r.unwrap();
    assert_eq!(vmsize, testvmsize);

    server.reset();

    // Check error logic, but fail fast without re-trying.
    let client = retry::Client::try_new()
        .unwrap()
        .max_retries(0)
        .mock_base_url(server.url());
    azure::Azure::with_client(Some(client)).unwrap_err();
}

#[test]
fn test_goalstate_certs() {
    let mut server = mockito::Server::new();
    let m_version = mock_fab_version(&mut server);
    let m_goalstate = mock_goalstate(&mut server, true);

    let client = retry::Client::try_new()
        .unwrap()
        .mock_base_url(server.url());
    let provider = azure::Azure::with_client(Some(client)).unwrap();
    let goalstate = provider.fetch_goalstate().unwrap();

    m_version.assert();
    m_goalstate.assert();

    let ep = goalstate.certs_endpoint().unwrap();
    let certs_url = reqwest::Url::parse(&ep).unwrap();
    assert_eq!(certs_url.scheme(), "http");

    server.reset();
}

#[test]
fn test_goalstate_no_certs() {
    let mut server = mockito::Server::new();
    let m_version = mock_fab_version(&mut server);
    let m_goalstate = mock_goalstate(&mut server, false);

    let client = retry::Client::try_new()
        .unwrap()
        .mock_base_url(server.url());
    let provider = azure::Azure::with_client(Some(client)).unwrap();
    let goalstate = provider.fetch_goalstate().unwrap();

    m_version.assert();
    m_goalstate.assert();

    assert_eq!(goalstate.certs_endpoint(), None);

    server.reset();
}
