<script lang="js">
  import { InMemoryCache } from '@apollo/client/cache';
  import { ApolloClient } from '@apollo/client/core';
  import { WebSocketLink } from '@apollo/client/link/ws';
  import { SubscriptionClient } from 'subscriptions-transport-ws';
  import { onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { setClient, subscribe } from 'svelte-apollo';
  import Router, { replace, location, link } from 'svelte-spa-router';
  import { wrap } from 'svelte-spa-router/wrap';

  import {
    ExportAllRestreams,
    Info,
    State,
  } from './api/graphql/client.graphql';

  import { showError, isOutputPage } from './util';

  import UIkit from 'uikit';
  import Icons from 'uikit/dist/js/uikit-icons';

  import { restreamModal, exportModal } from './stores';

  import RestreamModal from './RestreamModal.svelte';
  import OutputModal from './OutputModal.svelte';
  import PasswordModal from './PasswordModal.svelte';
  import ExportModal from './ExportModal.svelte';

  import * as PageAll from './pages/All.svelte';
  import * as PageOutput from './pages/Output.svelte';
  import TitleModal from './TitleModal.svelte';

  UIkit.use(Icons);

  let isOnline = true;

  const wsClient = new SubscriptionClient(
    !!process.env.WEBPACK_DEV_SERVER
      ? 'ws://127.0.0.1/api'
      : 'ws' +
        (window.location.protocol === 'https:' ? 's' : '') +
        '://' +
        window.location.host +
        window.location.pathname.replace(/\/?$/g, '') +
        '/api',
    { reconnect: true }
  );
  wsClient.onConnected(() => {
    isOnline = true;
  });
  wsClient.onReconnected(() => {
    isOnline = true;
  });
  wsClient.onDisconnected(() => (isOnline = false));
  const gqlClient = new ApolloClient({
    link: new WebSocketLink(wsClient),
    cache: new InMemoryCache(),
  });
  setClient(gqlClient);

  const info = subscribe(Info, { errorPolicy: 'all' });
  const state = subscribe(State, { errorPolicy: 'all' });

  let currentHash = undefined;
  onDestroy(
    info.subscribe((i) => {
      if (i.data) {
        const newHash = i.data.info.passwordHash;
        if (currentHash === undefined) {
          currentHash = newHash;
        } else if (!!newHash && newHash !== currentHash) {
          window.location.reload();
        }

        const title = i.data.info.title;
        document.title = title || 'Ephyr re-streamer';
      }
    })
  );

  let openPasswordModal = false;
  let openTitleModal = false;

  async function openExportModal() {
    let resp;
    try {
      resp = await gqlClient.query({
        query: ExportAllRestreams,
        fetchPolicy: 'no-cache',
      });
    } catch (e) {
      showError(e.message);
      return;
    }

    if (!!resp.data) {
      exportModal.open(
        null,
        resp.data.export
          ? JSON.stringify(JSON.parse(resp.data.export), null, 2)
          : ''
      );
    }
  }

  const routes = {
    '/restream/:restream_id/output/:output_id': wrap({
      component: PageOutput,
      props: {
        state,
      },
      userData: {
        state,
      },
      conditions: [
        (detail) => {
          const st = get(detail.userData.state);
          const p = detail.location.split('/');
          return (
            !!st.data &&
            st.data.allRestreams.some(
              (r) => r.id === p[2] && r.outputs.some((o) => o.id === p[4])
            )
          );
        },
      ],
    }),
    '*': wrap({
      component: PageAll,
      props: {
        info,
        state,
      },
    }),
  };
</script>

<template>
  <header class="uk-container">
    {#if isOnline && $info.data}
      {#if !isOutputPage($location)}
        <button
          class="uk-button uk-button-primary"
          on:click={() => restreamModal.openAdd()}
        >
          <i class="fas fa-plus" />&nbsp;<span>Input</span>
        </button>
      {/if}
      {#if isOutputPage($location)}
        <a
          href="/"
          class="back-to-all"
          use:link
          title="Return back to all definitions"
        >
          <i class="fas fa-th" />
        </a>
      {/if}
      {#key $info.data.info.passwordHash}
        <a
          href="/"
          class="set-password"
          on:click|preventDefault={() => (openPasswordModal = true)}
        >
          <i
            class="fas"
            class:fa-lock-open={!$info.data.info.passwordHash}
            class:fa-lock={!!$info.data.info.passwordHash}
            title="{!$info.data.info.passwordHash ? 'Set' : 'Change'} password"
          />
        </a>
      {/key}
      <TitleModal
        title={$info.data.info.title}
        bind:visible={openTitleModal}
      />
      <PasswordModal
        current_hash={$info.data.info.passwordHash}
        bind:visible={openPasswordModal}
      />
      <OutputModal />
      {#if !isOutputPage($location)}
        <RestreamModal public_host={$info.data.info.publicHost} />
        {#if isOnline && $state.data}
          <ExportModal />
          <a
            class="export-import-all"
            href="/"
            on:click|preventDefault={openExportModal}
            title="Export/Import all"
          >
            <i class="fas fa-share-square" />
          </a>
        {/if}
      {/if}
    {:else if $info.error}
      {showError($info.error.message) || ''}
    {/if}

    <a
      href="https://allatraunites.com"
      target="_blank"
      class="logo"
      title="Join us on allatraunites.com"
    >
      <img src="logo.jpg" alt="Logo" />
      <h3>Creative Society</h3>
      <small>Ephyr re-streamer {process.env.VERSION}</small>
    </a>
    <a
      href="#"
      class="set-title"
      on:click|preventDefault={() => (openTitleModal = true)}
    >
      <i class="fas fa-edit" title="Set title" />
    </a>
  </header>

  <main class="uk-container">
    {#if !isOnline || $state.loading}
      <div class="uk-alert uk-alert-warning loading">Loading...</div>
    {:else if isOnline && $state.data && $info.data}
      <Router {routes} on:conditionsFailed={() => replace('/')} />
    {/if}
    {#if $state.error}
      {showError($state.error.message) || ''}
    {/if}
  </main>

  <footer class="uk-container">
    Developed for people with ❤ by
    <a href="https://github.com/ALLATRA-IT" target="_blank">AllatRa IT</a>
  </footer>
</template>

<style lang="stylus" global>
  @require "../node_modules/uikit/dist/css/uikit.min.css"

  $header_height = 64px
  $footer_height = 30px

  h2, h3
    color: #666

  .uk-container
    padding-left: 34px !important
    padding-right: @padding-left

  header
    position: relative
    padding: 10px
    height: $header_height - 2 * @padding

    button
      float: right
    .set-password, .set-title
      margin-right: 30px
    .back-to-all
      margin-top: 2px
    .set-password, .set-title, .back-to-all
      float: right
      font-size: 26px
      color: #666
      outline: none
      &:hover
        text-decoration: none
        color: #444

    .logo
      outline: none
      position: relative
      white-space: nowrap
      display: inline-block

      &:hover
        text-decoration: none

      img
        width: 44px
        height: @width
        float: left

      h3
        margin: 4px 0 4px 52px
        max-width: 50%

      small
        position: absolute
        font-size: 12px
        width: 200px
        bottom: -6px
        left: 68px
        color: #999

    .export-import-all
      position: absolute
      top: 18px
      right: 9px
      opacity: 0
      transition: opacity .3s ease
      color: #666
      outline: none

      &:hover
        text-decoration: none
        color: #444
        opacity: 1

    &:hover
      .export-import-all
        opacity: 1

  main
    min-height: "calc(100vh - %s)" % ($header_height + $footer_height)

    > .loading
      text-align: center

  .uk-button-primary
    background-color: #08c

    &:not([disabled]):hover
      background-color: #046

  footer
    padding-top: 10px
    height: $footer_height - @padding-top
    font-size: 12px
</style>
