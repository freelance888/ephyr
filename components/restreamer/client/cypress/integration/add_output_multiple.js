describe('ADD MULTIPLE OUTPUT', () => {
  it('Goes to the homepage', () => {
    cy.visit('/');
  });
  it('Click Output', () => {
    cy.get("span:contains('Output'):first").click();
    cy.wait(5000);
  });

  it('Click Multiple - Json', () => {
    cy.get("a:contains('Multiple - Json')").click();
  });

  it('Pastes text to textarea', () => {
    const textToPaste =
      '{"label":"[Manual Start] FB АЛЛАТРА ТВ България / @valeranedov","url":"rtmps://live-api-s.facebook.com:443/rtmp/FB-348459623681583-","preview_url":"https://www.facebook.com/allatratvbulgaria/posts/348462730347939"},{"label":"[Manual Start] YT Съзидателно Общество. AllatraUnites / @valeranedov","url":"rtmp://a.rtmp.youtube.com/live2/rwhetk-2s44","preview_url":"https://youtu.be/jKl7txehM78"}';
    cy.get('.multi-json-form > .uk-textarea')
      .invoke('val', textToPaste)
      .trigger('input');
  });

  it('Submits', () => {
    cy.get("button:contains('Add')").click();
    cy.get("button:contains('Add')").should('not.exist');
  });

  it('Assert', () => {
    cy.get("span:contains('[Manual Start] FB')").should(
      'have.text',
      '[Manual Start] FB АЛЛАТРА ТВ България / @valeranedov'
    );
    cy.get("span:contains('[Manual Start] YT')").should(
      'have.text',
      '[Manual Start] YT Съзидателно Общество. AllatraUnites / @valeranedov'
    );
    cy.get("span:contains('rtmps://live-api')").should(
      'have.text',
      'rtmps://live-api-s.facebook.com:443/rtmp/FB-348459623681583-'
    );
    cy.get("span:contains('rtmp://a.rtmp')").should(
      'have.text',
      'rtmp://a.rtmp.youtube.com/live2/rwhetk-2s44'
    );
  });
});
