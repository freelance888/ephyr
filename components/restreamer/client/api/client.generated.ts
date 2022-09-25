import client from './apollo-client';
import type {
  ApolloQueryResult,
  ObservableQuery,
  WatchQueryOptions,
  MutationOptions,
  SubscriptionOptions,
} from '@apollo/client';
import { readable } from 'svelte/store';
import type { Readable } from 'svelte/store';
import gql from 'graphql-tag';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = {
  [K in keyof T]: T[K];
};
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & {
  [SubKey in K]?: Maybe<T[SubKey]>;
};
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & {
  [SubKey in K]: Maybe<T[SubKey]>;
};
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  Delay: any;
  EndpointId: any;
  InputId: any;
  InputKey: any;
  InputSrcUrl: any;
  Label: any;
  MixinId: any;
  MixinSrcUrl: any;
  OutputDstUrl: any;
  OutputId: any;
  RestreamId: any;
  RestreamKey: any;
  Url: any;
  VolumeLevel: any;
};

/** Backup input */
export type BackupInput = {
  /** Key */
  key: Scalars['InputKey'];
  /** URL to pull a live stream from for a backup endpoint. */
  src?: InputMaybe<Scalars['InputSrcUrl']>;
};

/**
 * Failover source of multiple `Input`s to pull a live stream by an `Input`
 * from.
 */
export type FailoverInputSrc = {
  __typename?: 'FailoverInputSrc';
  /**
   * `Input`s forming this `FailoverInputSrc`.
   *
   * Failover is implemented by attempting to pull the first `Input` falling
   * back to the second one, and so on. Once the first source is restored,
   * we pool from it once again.
   */
  inputs: Array<Input>;
};

/** Information about parameters that this server operates with. */
export type Info = {
  __typename?: 'Info';
  /** Whether do we need to confirm deletion of inputs and outputs */
  deleteConfirmation?: Maybe<Scalars['Boolean']>;
  /** Whether do we need to confirm enabling/disabling of inputs or outputs */
  enableConfirmation?: Maybe<Scalars['Boolean']>;
  /**
   * [Argon2] hash of the password that this server's GraphQL API is
   * protected with, if any.
   *
   * Non-`null` value means that any request to GraphQL API should perform
   * [HTTP Basic auth][1]. Any username is allowed, but the password should
   * match this hash.
   *
   * [Argon2]: https://en.wikipedia.org/wiki/Argon2
   * [1]: https://en.wikipedia.org/wiki/Basic_access_authentication
   */
  passwordHash?: Maybe<Scalars['String']>;
  /** Password hash for single output application */
  passwordOutputHash?: Maybe<Scalars['String']>;
  /**
   * Host that this server is reachable via in public.
   *
   * Use it for constructing URLs to this server.
   */
  publicHost: Scalars['String'];
  /** Title of the server */
  title?: Maybe<Scalars['String']>;
};

/** Upstream source that a `Restream` receives a live stream from. */
export type Input = {
  __typename?: 'Input';
  /**
   * Indicator whether this `Input` is enabled, so is allowed to receive a
   * live stream from its upstream sources.
   */
  enabled: Scalars['Boolean'];
  /**
   * Endpoints of this `Input` serving a live stream for `Output`s and
   * clients.
   */
  endpoints: Array<InputEndpoint>;
  /**
   * Unique ID of this `Input`.
   *
   * Once assigned, it never changes.
   */
  id: Scalars['InputId'];
  /**
   * Key of this `Input` to expose its `InputEndpoint`s with for accepting
   * and serving a live stream.
   */
  key: Scalars['InputKey'];
  /**
   * Source to pull a live stream from.
   *
   * If specified, then this `Input` will pull a live stream from it (pull
   * kind), otherwise this `Input` will await a live stream to be pushed
   * (push kind).
   */
  src?: Maybe<InputSrc>;
};

/** Endpoint of an `Input` serving a live stream for `Output`s and clients. */
export type InputEndpoint = {
  __typename?: 'InputEndpoint';
  /**
   * Unique ID of this `InputEndpoint`.
   *
   * Once assigned, it never changes.
   */
  id: Scalars['EndpointId'];
  /** Kind of this `InputEndpoint`. */
  kind: InputEndpointKind;
  /** User defined label for each Endpoint */
  label?: Maybe<Scalars['Label']>;
  /**
   * `Status` of this `InputEndpoint` indicating whether it actually serves a
   * live stream ready to be consumed by `Output`s and clients.
   */
  status: Status;
};

/** Possible kinds of an `InputEndpoint`. */
export enum InputEndpointKind {
  /**
   * [HLS] endpoint.
   *
   * Only serves a live stream for playing and is not able to accept one.
   *
   * [HLS]: https://en.wikipedia.org/wiki/HTTP_Live_Streaming
   */
  Hls = 'HLS',
  /**
   * [RTMP] endpoint.
   *
   * Can accept a live stream and serve it for playing.
   *
   * [RTMP]: https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol
   */
  Rtmp = 'RTMP',
}

/** Source to pull a live stream by an `Input` from. */
export type InputSrc = FailoverInputSrc | RemoteInputSrc;

/**
 * Additional source for an `Output` to be mixed with before re-streaming to
 * the destination.
 */
export type Mixin = {
  __typename?: 'Mixin';
  /**
   * Delay that this `Mixin` should wait before being mixed with an `Output`.
   *
   * Very useful to fix de-synchronization issues and correct timings between
   * a `Mixin` and its `Output`.
   */
  delay: Scalars['Delay'];
  /**
   * Unique ID of this `Mixin`.
   *
   * Once assigned, it never changes.
   */
  id: Scalars['MixinId'];
  /**
   * Side-chain audio of `Output` with this `Mixin`.
   *
   * Helps to automatically control audio level of `Mixin`
   * based on level of `Output`.
   */
  sidechain: Scalars['Boolean'];
  /**
   * URL of the source to be mixed with an `Output`.
   *
   * At the moment, only [TeamSpeak] is supported.
   *
   * [TeamSpeak]: https://teamspeak.com
   */
  src: Scalars['MixinSrcUrl'];
  /**
   * `Status` of this `Mixin` indicating whether it provides an actual media
   * stream to be mixed with its `Output`.
   */
  status: Status;
  /** Volume rate of this `Mixin`'s audio tracks to mix them with. */
  volume: Volume;
};

export type Mutation = {
  __typename?: 'Mutation';
  /**
   * Sets an `Input`'s endpoint label by `Input` and `Endpoint` `id`.
   *
   * ### Result
   *
   * Returns `true` if the label has been set with the given `label`,
   * `false` if it was not
   * `null` if the `Input` or `Endpoint` doesn't exist.
   */
  changeEndpointLabel?: Maybe<Scalars['Boolean']>;
  /**
   * Disables all `Output`s in the specified `Restream`.
   *
   * Disabled `Output`s stop re-streaming a live stream to their
   * destinations.
   *
   * ### Result
   *
   * Returns `true` if at least one `Output` has been disabled, `false` if
   * all `Output`s have been disabled already, and `null` if the specified
   * `Restream` doesn't exist.
   */
  disableAllOutputs?: Maybe<Scalars['Boolean']>;
  /**
   * Disables all `Output`s in all `Restream`s.
   *
   * Disabled `Output`s stop re-streaming a live stream to their
   * destinations.
   *
   * ### Result
   *
   * Returns `true` if at least one `Output` has been disabled, `false` if
   * all `Output`s have been disabled already or there are no outputs
   */
  disableAllOutputsOfRestreams: Scalars['Boolean'];
  /**
   * Disables an `Input` by its `id`.
   *
   * Disabled `Input` stops all on-going re-streaming processes and is not
   * allowed to accept or pull a live stream.
   *
   * ### Result
   *
   * Returns `true` if an `Input` with the given `id` has been disabled,
   * `false` if it has been disabled already, and `null` if it doesn't exist.
   */
  disableInput?: Maybe<Scalars['Boolean']>;
  /**
   * Disables an `Output` by its `id` in the specified `Restream`.
   *
   * Disabled `Output` stops re-streaming a live stream to its destination.
   *
   * ### Result
   *
   * Returns `true` if an `Output` with the given `id` has been disabled,
   * `false` if it has been disabled already, and `null` if the specified
   * `Restream`/`Output` doesn't exist.
   */
  disableOutput?: Maybe<Scalars['Boolean']>;
  /**
   * Disables a `Restream` by its `id`.
   *
   * Disabled `Restream` stops all on-going re-streaming processes and is not
   * allowed to accept or pull a live stream.
   *
   * ### Result
   *
   * Returns `true` if a `Restream` with the given `id` has been disabled,
   * `false` if it has been disabled already, and `null` if it doesn't exist.
   */
  disableRestream?: Maybe<Scalars['Boolean']>;
  /**
   * Enables all `Output`s in the specified `Restream`.
   *
   * Enabled `Output`s start re-streaming a live stream to their
   * destinations.
   *
   * ### Result
   *
   * Returns `true` if at least one `Output` has been enabled, `false` if all
   * `Output`s have been enabled already, and `null` if the specified
   * `Restream` doesn't exist.
   */
  enableAllOutputs?: Maybe<Scalars['Boolean']>;
  /**
   * Enables an `Input` by its `id`.
   *
   * Enabled `Input` is allowed to accept or pull a live stream.
   *
   * ### Result
   *
   * Returns `true` if an `Input` with the given `id` has been enabled,
   * `false` if it has been enabled already, and `null` if it doesn't exist.
   */
  enableInput?: Maybe<Scalars['Boolean']>;
  /**
   * Enables an `Output` by its `id` in the specified `Restream`.
   *
   * Enabled `Output` starts re-streaming a live stream to its destination.
   *
   * ### Result
   *
   * Returns `true` if an `Output` with the given `id` has been enabled,
   * `false` if it has been enabled already, and `null` if the specified
   * `Restream`/`Output` doesn't exist.
   */
  enableOutput?: Maybe<Scalars['Boolean']>;
  /**
   * Enables a `Restream` by its `id`.
   *
   * Enabled `Restream` is allowed to accept or pull a live stream.
   *
   * ### Result
   *
   * Returns `true` if a `Restream` with the given `id` has been enabled,
   * `false` if it has been enabled already, and `null` if it doesn't exist.
   */
  enableRestream?: Maybe<Scalars['Boolean']>;
  /**
   * Enables all `Output`s in all `Restream`s.
   *
   * Enabled `Output`s start re-streaming a live stream to their
   * destinations.
   *
   * ### Result
   *
   * Returns `true` if at least one `Output` has been enabled, `false` if all
   * `Output`s have been enabled already or there are no outputs
   */
  enablesAllOutputsOfRestreams: Scalars['Boolean'];
  /**
   * Applies the specified JSON `spec` of `Restream`s to this server.
   *
   * If `replace` is `true` then replaces all the existing `Restream`s with
   * the one defined by the `spec`. Otherwise, merges the `spec` with
   * existing `Restream`s.
   *
   * ### Result
   *
   * Returns `null` if a `Restream` with the given `id` doesn't exist,
   * otherwise always returns `true`.
   */
  import?: Maybe<Scalars['Boolean']>;
  /**
   * Removes the specified recorded file.
   *
   * ### Result
   *
   * Returns `true` if the specified recorded file was removed, otherwise
   * `false` if nothing changes.
   */
  removeDvrFile: Scalars['Boolean'];
  /**
   * Removes an `Output` by its `id` from the specified `Restream`.
   *
   * ### Result
   *
   * Returns `null` if the specified `Restream`/`Output` doesn't exist,
   * otherwise always returns `true`.
   */
  removeOutput?: Maybe<Scalars['Boolean']>;
  /**
   * Removes a `Restream` by its `id`.
   *
   * ### Result
   *
   * Returns `null` if `Restream` with the given `id` doesn't exist,
   * otherwise always returns `true`.
   */
  removeRestream?: Maybe<Scalars['Boolean']>;
  /**
   * Sets a new `Output` or updates an existing one (if `id` is specified).
   *
   * ### Idempotency
   *
   * Idempotent if `id` is specified. Otherwise is non-idempotent, always
   * creates a new `Output` and errors on the `dst` duplicates within the
   * specified `Restream`.
   *
   * ### Result
   *
   * Returns `null` if a `Restream` with the given `restreamId` doesn't
   * exist, or an `Output` with the given `id` doesn't exist, otherwise
   * always returns `true`.
   */
  setOutput?: Maybe<Scalars['Boolean']>;
  /**
   * Sets or unsets the password to protect this GraphQL API with.
   *
   * Once password is set, any subsequent requests to this GraphQL API should
   * perform [HTTP Basic auth][1], where any username is allowed, but the
   * password should match the one being set.
   *
   * ### Result
   *
   * Returns `true` if password has been changed or unset, otherwise `false`
   * if nothing changes.
   *
   * [1]: https://en.wikipedia.org/wiki/Basic_access_authentication
   */
  setPassword: Scalars['Boolean'];
  /**
   * Sets a new `Restream` or updates an existing one (if `id` is specified).
   *
   * ### Idempotency
   *
   * Idempotent if `id` is specified. Otherwise is non-idempotent, always
   * creates a new `Restream` and errors on the `key` duplicates.
   *
   * ### Result
   *
   * Returns `null` if a `Restream` with the given `id` doesn't exist,
   * otherwise always returns `true`.
   */
  setRestream?: Maybe<Scalars['Boolean']>;
  /**
   * Sets settings of the server
   *
   * ### Result
   *
   * Returns `false` if title does not pass validation for max allowed
   * characters length. Otherwise returns `true`
   */
  setSettings: Scalars['Boolean'];
  /**
   * Tunes a `Delay` of the specified `Mixin` before mix it into its
   * `Output`.
   *
   * ### Result
   *
   * Returns `true` if a `Delay` has been changed, `false` if it has the same
   * value already, or `null` if the specified `Output` or `Mixin` doesn't
   * exist.
   */
  tuneDelay?: Maybe<Scalars['Boolean']>;
  /**
   * Tunes a `Sidechain` of the specified `Mixin` before mix it into its
   * `Output`.
   *
   * ### Result
   *
   * Returns `true` if a `Sidechain` has been changed, `false` if it has
   * the same value already, or `null` if the specified `Output`
   * or `Mixin` doesn't exist.
   */
  tuneSidechain?: Maybe<Scalars['Boolean']>;
  /**
   * Tunes a `Volume` rate of the specified `Output` or one of its `Mixin`s.
   *
   * ### Result
   *
   * Returns `true` if a `Volume` rate has been changed, `false` if it has
   * the same value already, or `null` if the specified `Output` or `Mixin`
   * doesn't exist.
   */
  tuneVolume?: Maybe<Scalars['Boolean']>;
};

export type MutationChangeEndpointLabelArgs = {
  endpointId: Scalars['EndpointId'];
  id: Scalars['InputId'];
  label: Scalars['String'];
  restreamId: Scalars['RestreamId'];
};

export type MutationDisableAllOutputsArgs = {
  restreamId: Scalars['RestreamId'];
};

export type MutationDisableInputArgs = {
  id: Scalars['InputId'];
  restreamId: Scalars['RestreamId'];
};

export type MutationDisableOutputArgs = {
  id: Scalars['OutputId'];
  restreamId: Scalars['RestreamId'];
};

export type MutationDisableRestreamArgs = {
  id: Scalars['RestreamId'];
};

export type MutationEnableAllOutputsArgs = {
  restreamId: Scalars['RestreamId'];
};

export type MutationEnableInputArgs = {
  id: Scalars['InputId'];
  restreamId: Scalars['RestreamId'];
};

export type MutationEnableOutputArgs = {
  id: Scalars['OutputId'];
  restreamId: Scalars['RestreamId'];
};

export type MutationEnableRestreamArgs = {
  id: Scalars['RestreamId'];
};

export type MutationImportArgs = {
  replace?: Scalars['Boolean'];
  restreamId?: InputMaybe<Scalars['RestreamId']>;
  spec: Scalars['String'];
};

export type MutationRemoveDvrFileArgs = {
  path: Scalars['String'];
};

export type MutationRemoveOutputArgs = {
  id: Scalars['OutputId'];
  restreamId: Scalars['RestreamId'];
};

export type MutationRemoveRestreamArgs = {
  id: Scalars['RestreamId'];
};

export type MutationSetOutputArgs = {
  dst: Scalars['OutputDstUrl'];
  id?: InputMaybe<Scalars['OutputId']>;
  label?: InputMaybe<Scalars['Label']>;
  mixins?: Array<Scalars['MixinSrcUrl']>;
  previewUrl?: InputMaybe<Scalars['Url']>;
  restreamId: Scalars['RestreamId'];
};

export type MutationSetPasswordArgs = {
  kind?: InputMaybe<PasswordKind>;
  new?: InputMaybe<Scalars['String']>;
  old?: InputMaybe<Scalars['String']>;
};

export type MutationSetRestreamArgs = {
  backupInputs?: InputMaybe<Array<BackupInput>>;
  id?: InputMaybe<Scalars['RestreamId']>;
  key: Scalars['RestreamKey'];
  label?: InputMaybe<Scalars['Label']>;
  src?: InputMaybe<Scalars['InputSrcUrl']>;
  withHls?: Scalars['Boolean'];
};

export type MutationSetSettingsArgs = {
  deleteConfirmation?: InputMaybe<Scalars['Boolean']>;
  enableConfirmation?: InputMaybe<Scalars['Boolean']>;
  title?: InputMaybe<Scalars['String']>;
};

export type MutationTuneDelayArgs = {
  delay: Scalars['Delay'];
  mixinId: Scalars['MixinId'];
  outputId: Scalars['OutputId'];
  restreamId: Scalars['RestreamId'];
};

export type MutationTuneSidechainArgs = {
  mixinId: Scalars['MixinId'];
  outputId: Scalars['OutputId'];
  restreamId: Scalars['RestreamId'];
  sidechain: Scalars['Boolean'];
};

export type MutationTuneVolumeArgs = {
  level: Scalars['VolumeLevel'];
  mixinId?: InputMaybe<Scalars['MixinId']>;
  muted: Scalars['Boolean'];
  outputId: Scalars['OutputId'];
  restreamId: Scalars['RestreamId'];
};

/** Downstream destination that a `Restream` re-streams a live stream to. */
export type Output = {
  __typename?: 'Output';
  /**
   * Downstream URL to re-stream a live stream onto.
   *
   * At the moment only [RTMP] and [Icecast] are supported.
   *
   * [Icecast]: https://icecast.org
   * [RTMP]: https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol
   */
  dst: Scalars['OutputDstUrl'];
  /**
   * Indicator whether this `Output` is enabled, so is allowed to perform a
   * live stream re-streaming to its downstream destination.
   */
  enabled: Scalars['Boolean'];
  /**
   * Unique ID of this `Output`.
   *
   * Once assigned, it never changes.
   */
  id: Scalars['OutputId'];
  /** Optional label of this `Output`. */
  label?: Maybe<Scalars['Label']>;
  /**
   * `Mixin`s to mix this `Output` with before re-streaming it to its
   * downstream destination.
   *
   * If empty, then no mixing is performed and re-streaming is as cheap as
   * possible (just copies bytes "as is").
   */
  mixins: Array<Mixin>;
  /** Url of stream preview. */
  previewUrl?: Maybe<Scalars['Url']>;
  /**
   * `Status` of this `Output` indicating whether it actually re-streams a
   * live stream to its downstream destination.
   */
  status: Status;
  /**
   * Volume rate of this `Output`'s audio tracks when mixed with
   * `Output.mixins`.
   *
   * Has no effect when there is no `Output.mixins`.
   */
  volume: Volume;
};

/** Specifies kind of password */
export enum PasswordKind {
  /** Password for main application */
  Main = 'MAIN',
  /** Password for single output application */
  Output = 'OUTPUT',
}

export type Query = {
  __typename?: 'Query';
  /** Returns all the `Restream`s happening on this server. */
  allRestreams: Array<Restream>;
  /**
   * Returns list of recorded files of the specified `Output`.
   *
   * If returned list is empty, the there is no recorded files for the
   * specified `Output`.
   *
   * Each recorded file is represented as a relative path on [SRS] HTTP
   * server in `dvr/` directory, so the download link should look like this:
   * ```ignore
   * http://my.host:8080/dvr/returned/file/path.flv
   * ```
   *
   * [SRS]: https://github.com/ossrs/srs
   */
  dvrFiles: Array<Scalars['String']>;
  /**
   * Returns `Restream`s happening on this server and identifiable by the
   * given `ids` in an exportable JSON format.
   *
   * If no `ids` specified, then returns all the `Restream`s happening on
   * this server at the moment.
   */
  export?: Maybe<Scalars['String']>;
  /** Returns the current `Info` parameters of this server. */
  info: Info;
  /** Returns the current `ServerInfo` */
  serverInfo: ServerInfo;
};

export type QueryDvrFilesArgs = {
  id: Scalars['OutputId'];
};

export type QueryExportArgs = {
  ids?: Array<Scalars['RestreamId']>;
};

/** Remote upstream source to pull a live stream by an `Input` from. */
export type RemoteInputSrc = {
  __typename?: 'RemoteInputSrc';
  /** Label for this Endpoint */
  label?: Maybe<Scalars['Label']>;
  /** URL of this `RemoteInputSrc`. */
  url: Scalars['InputSrcUrl'];
};

/** Re-stream of a live stream from one `Input` to many `Output`s. */
export type Restream = {
  __typename?: 'Restream';
  /**
   * Unique ID of this `Input`.
   *
   * Once assigned, it never changes.
   */
  id: Scalars['RestreamId'];
  /** `Input` that a live stream is received from. */
  input: Input;
  /**
   * Unique key of this `Restream` identifying it, and used to form its
   * endpoints URLs.
   */
  key: Scalars['RestreamKey'];
  /** Optional label of this `Restream`. */
  label?: Maybe<Scalars['Label']>;
  /** `Output`s that a live stream is re-streamed to. */
  outputs: Array<Output>;
};

/** Server's info */
export type ServerInfo = {
  __typename?: 'ServerInfo';
  /** Total CPU usage, % */
  cpuUsage?: Maybe<Scalars['Float']>;
  /** Error message */
  errorMsg?: Maybe<Scalars['String']>;
  /** Free (available) RAM */
  ramFree?: Maybe<Scalars['Float']>;
  /** Total RAM installed on current machine */
  ramTotal?: Maybe<Scalars['Float']>;
  /** Network traffic, received last second */
  rxDelta?: Maybe<Scalars['Float']>;
  /** Network traffic, transferred last second */
  txDelta?: Maybe<Scalars['Float']>;
};

/** Status indicating availability of an `Input`, `Output`, or a `Mixin`. */
export enum Status {
  /** Initializing, media traffic doesn't yet flow as expected. */
  Initializing = 'INITIALIZING',
  /** Inactive, no operations are performed and no media traffic is flowed. */
  Offline = 'OFFLINE',
  /**
   * Active, all operations are performing successfully and media traffic
   * flows as expected.
   */
  Online = 'ONLINE',
  /** Failed recently */
  Unstable = 'UNSTABLE',
}

export type Subscription = {
  __typename?: 'Subscription';
  /** Subscribes to updates of all `Restream`s happening on this server. */
  allRestreams: Array<Restream>;
  /** Subscribes to updates of `Info` parameters of this server. */
  info: Info;
  /** Subscribes to updates of `ServerInfo` parameters of this server. */
  serverInfo: ServerInfo;
};

/** Volume rate of an audio track in percents and flag if it is muted. */
export type Volume = {
  __typename?: 'Volume';
  /** Volume rate or level */
  level: Scalars['VolumeLevel'];
  /** Whether it is muted or not */
  muted: Scalars['Boolean'];
};

export type InfoSubscriptionVariables = Exact<{ [key: string]: never }>;

export type InfoSubscription = {
  __typename?: 'Subscription';
  info: {
    __typename?: 'Info';
    publicHost: string;
    title?: string | null;
    deleteConfirmation?: boolean | null;
    enableConfirmation?: boolean | null;
    passwordHash?: string | null;
    passwordOutputHash?: string | null;
  };
};

export type ServerInfoSubscriptionVariables = Exact<{ [key: string]: never }>;

export type ServerInfoSubscription = {
  __typename?: 'Subscription';
  serverInfo: {
    __typename?: 'ServerInfo';
    cpuUsage?: number | null;
    ramTotal?: number | null;
    ramFree?: number | null;
    txDelta?: number | null;
    rxDelta?: number | null;
    errorMsg?: string | null;
  };
};

export type StateSubscriptionVariables = Exact<{ [key: string]: never }>;

export type StateSubscription = {
  __typename?: 'Subscription';
  allRestreams: Array<{
    __typename?: 'Restream';
    id: any;
    key: any;
    label?: any | null;
    input: {
      __typename?: 'Input';
      id: any;
      key: any;
      enabled: boolean;
      endpoints: Array<{
        __typename?: 'InputEndpoint';
        id: any;
        kind: InputEndpointKind;
        status: Status;
        label?: any | null;
      }>;
      src?:
        | {
            __typename?: 'FailoverInputSrc';
            inputs: Array<{
              __typename?: 'Input';
              id: any;
              key: any;
              enabled: boolean;
              endpoints: Array<{
                __typename?: 'InputEndpoint';
                id: any;
                kind: InputEndpointKind;
                status: Status;
                label?: any | null;
              }>;
              src?:
                | { __typename?: 'FailoverInputSrc' }
                | {
                    __typename?: 'RemoteInputSrc';
                    url: any;
                    label?: any | null;
                  }
                | null;
            }>;
          }
        | { __typename?: 'RemoteInputSrc'; url: any; label?: any | null }
        | null;
    };
    outputs: Array<{
      __typename?: 'Output';
      id: any;
      dst: any;
      label?: any | null;
      previewUrl?: any | null;
      enabled: boolean;
      status: Status;
      volume: { __typename?: 'Volume'; level: any; muted: boolean };
      mixins: Array<{
        __typename?: 'Mixin';
        id: any;
        src: any;
        delay: any;
        sidechain: boolean;
        volume: { __typename?: 'Volume'; level: any; muted: boolean };
      }>;
    }>;
  }>;
};

export type DvrFilesQueryVariables = Exact<{
  id: Scalars['OutputId'];
}>;

export type DvrFilesQuery = { __typename?: 'Query'; dvrFiles: Array<string> };

export type ExportRestreamQueryVariables = Exact<{
  id: Scalars['RestreamId'];
}>;

export type ExportRestreamQuery = {
  __typename?: 'Query';
  export?: string | null;
};

export type ExportAllRestreamsQueryVariables = Exact<{ [key: string]: never }>;

export type ExportAllRestreamsQuery = {
  __typename?: 'Query';
  export?: string | null;
};

export type ImportMutationVariables = Exact<{
  restream_id?: InputMaybe<Scalars['RestreamId']>;
  replace: Scalars['Boolean'];
  spec: Scalars['String'];
}>;

export type ImportMutation = {
  __typename?: 'Mutation';
  import?: boolean | null;
};

export type SetRestreamMutationVariables = Exact<{
  key: Scalars['RestreamKey'];
  url?: InputMaybe<Scalars['InputSrcUrl']>;
  label?: InputMaybe<Scalars['Label']>;
  id?: InputMaybe<Scalars['RestreamId']>;
  backup_inputs?: InputMaybe<Array<BackupInput> | BackupInput>;
  with_hls: Scalars['Boolean'];
}>;

export type SetRestreamMutation = {
  __typename?: 'Mutation';
  setRestream?: boolean | null;
};

export type RemoveRestreamMutationVariables = Exact<{
  id: Scalars['RestreamId'];
}>;

export type RemoveRestreamMutation = {
  __typename?: 'Mutation';
  removeRestream?: boolean | null;
};

export type EnableInputMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
  input_id: Scalars['InputId'];
}>;

export type EnableInputMutation = {
  __typename?: 'Mutation';
  enableInput?: boolean | null;
};

export type DisableInputMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
  input_id: Scalars['InputId'];
}>;

export type DisableInputMutation = {
  __typename?: 'Mutation';
  disableInput?: boolean | null;
};

export type SetEndpointLabelMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
  input_id: Scalars['InputId'];
  endpoint_id: Scalars['EndpointId'];
  label: Scalars['String'];
}>;

export type SetEndpointLabelMutation = {
  __typename?: 'Mutation';
  changeEndpointLabel?: boolean | null;
};

export type SetOutputMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
  url: Scalars['OutputDstUrl'];
  label?: InputMaybe<Scalars['Label']>;
  preview_url?: InputMaybe<Scalars['Url']>;
  mixins: Array<Scalars['MixinSrcUrl']> | Scalars['MixinSrcUrl'];
  id?: InputMaybe<Scalars['OutputId']>;
}>;

export type SetOutputMutation = {
  __typename?: 'Mutation';
  setOutput?: boolean | null;
};

export type RemoveOutputMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
  output_id: Scalars['OutputId'];
}>;

export type RemoveOutputMutation = {
  __typename?: 'Mutation';
  removeOutput?: boolean | null;
};

export type EnableOutputMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
  output_id: Scalars['OutputId'];
}>;

export type EnableOutputMutation = {
  __typename?: 'Mutation';
  enableOutput?: boolean | null;
};

export type DisableOutputMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
  output_id: Scalars['OutputId'];
}>;

export type DisableOutputMutation = {
  __typename?: 'Mutation';
  disableOutput?: boolean | null;
};

export type EnableAllOutputsMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
}>;

export type EnableAllOutputsMutation = {
  __typename?: 'Mutation';
  enableAllOutputs?: boolean | null;
};

export type DisableAllOutputsMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
}>;

export type DisableAllOutputsMutation = {
  __typename?: 'Mutation';
  disableAllOutputs?: boolean | null;
};

export type EnableAllOutputsOfRestreamsMutationVariables = Exact<{
  [key: string]: never;
}>;

export type EnableAllOutputsOfRestreamsMutation = {
  __typename?: 'Mutation';
  enablesAllOutputsOfRestreams: boolean;
};

export type DisableAllOutputsOfRestreamsMutationVariables = Exact<{
  [key: string]: never;
}>;

export type DisableAllOutputsOfRestreamsMutation = {
  __typename?: 'Mutation';
  disableAllOutputsOfRestreams: boolean;
};

export type TuneVolumeMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
  output_id: Scalars['OutputId'];
  mixin_id?: InputMaybe<Scalars['MixinId']>;
  level: Scalars['VolumeLevel'];
  muted: Scalars['Boolean'];
}>;

export type TuneVolumeMutation = {
  __typename?: 'Mutation';
  tuneVolume?: boolean | null;
};

export type TuneDelayMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
  output_id: Scalars['OutputId'];
  mixin_id: Scalars['MixinId'];
  delay: Scalars['Delay'];
}>;

export type TuneDelayMutation = {
  __typename?: 'Mutation';
  tuneDelay?: boolean | null;
};

export type TuneSidechainMutationVariables = Exact<{
  restream_id: Scalars['RestreamId'];
  output_id: Scalars['OutputId'];
  mixin_id: Scalars['MixinId'];
  sidechain: Scalars['Boolean'];
}>;

export type TuneSidechainMutation = {
  __typename?: 'Mutation';
  tuneSidechain?: boolean | null;
};

export type RemoveDvrFileMutationVariables = Exact<{
  path: Scalars['String'];
}>;

export type RemoveDvrFileMutation = {
  __typename?: 'Mutation';
  removeDvrFile: boolean;
};

export type SetPasswordMutationVariables = Exact<{
  new?: InputMaybe<Scalars['String']>;
  old?: InputMaybe<Scalars['String']>;
  kind: PasswordKind;
}>;

export type SetPasswordMutation = {
  __typename?: 'Mutation';
  setPassword: boolean;
};

export type SetSettingsMutationVariables = Exact<{
  title?: InputMaybe<Scalars['String']>;
  deleteConfirmation: Scalars['Boolean'];
  enableConfirmation: Scalars['Boolean'];
}>;

export type SetSettingsMutation = {
  __typename?: 'Mutation';
  setSettings: boolean;
};

export const InfoDoc = gql`
  subscription Info {
    info {
      publicHost
      title
      deleteConfirmation
      enableConfirmation
      passwordHash
      passwordOutputHash
    }
  }
`;
export const ServerInfoDoc = gql`
  subscription ServerInfo {
    serverInfo {
      cpuUsage
      ramTotal
      ramFree
      txDelta
      rxDelta
      errorMsg
    }
  }
`;
export const StateDoc = gql`
  subscription State {
    allRestreams {
      id
      key
      label
      input {
        id
        key
        endpoints {
          id
          kind
          status
          label
        }
        src {
          ... on RemoteInputSrc {
            url
            label
          }
          ... on FailoverInputSrc {
            inputs {
              id
              key
              endpoints {
                id
                kind
                status
                label
              }
              src {
                ... on RemoteInputSrc {
                  url
                  label
                }
              }
              enabled
            }
          }
        }
        enabled
      }
      outputs {
        id
        dst
        label
        previewUrl
        volume {
          level
          muted
        }
        mixins {
          id
          src
          volume {
            level
            muted
          }
          delay
          sidechain
        }
        enabled
        status
      }
    }
  }
`;
export const DvrFilesDoc = gql`
  query DvrFiles($id: OutputId!) {
    dvrFiles(id: $id)
  }
`;
export const ExportRestreamDoc = gql`
  query ExportRestream($id: RestreamId!) {
    export(ids: [$id])
  }
`;
export const ExportAllRestreamsDoc = gql`
  query ExportAllRestreams {
    export
  }
`;
export const ImportDoc = gql`
  mutation Import(
    $restream_id: RestreamId
    $replace: Boolean!
    $spec: String!
  ) {
    import(restreamId: $restream_id, replace: $replace, spec: $spec)
  }
`;
export const SetRestreamDoc = gql`
  mutation SetRestream(
    $key: RestreamKey!
    $url: InputSrcUrl
    $label: Label
    $id: RestreamId
    $backup_inputs: [BackupInput!]
    $with_hls: Boolean!
  ) {
    setRestream(
      key: $key
      src: $url
      label: $label
      backupInputs: $backup_inputs
      withHls: $with_hls
      id: $id
    )
  }
`;
export const RemoveRestreamDoc = gql`
  mutation RemoveRestream($id: RestreamId!) {
    removeRestream(id: $id)
  }
`;
export const EnableInputDoc = gql`
  mutation EnableInput($restream_id: RestreamId!, $input_id: InputId!) {
    enableInput(id: $input_id, restreamId: $restream_id)
  }
`;
export const DisableInputDoc = gql`
  mutation DisableInput($restream_id: RestreamId!, $input_id: InputId!) {
    disableInput(id: $input_id, restreamId: $restream_id)
  }
`;
export const SetEndpointLabelDoc = gql`
  mutation SetEndpointLabel(
    $restream_id: RestreamId!
    $input_id: InputId!
    $endpoint_id: EndpointId!
    $label: String!
  ) {
    changeEndpointLabel(
      id: $input_id
      restreamId: $restream_id
      endpointId: $endpoint_id
      label: $label
    )
  }
`;
export const SetOutputDoc = gql`
  mutation SetOutput(
    $restream_id: RestreamId!
    $url: OutputDstUrl!
    $label: Label
    $preview_url: Url
    $mixins: [MixinSrcUrl!]!
    $id: OutputId
  ) {
    setOutput(
      restreamId: $restream_id
      dst: $url
      label: $label
      previewUrl: $preview_url
      mixins: $mixins
      id: $id
    )
  }
`;
export const RemoveOutputDoc = gql`
  mutation RemoveOutput($restream_id: RestreamId!, $output_id: OutputId!) {
    removeOutput(restreamId: $restream_id, id: $output_id)
  }
`;
export const EnableOutputDoc = gql`
  mutation EnableOutput($restream_id: RestreamId!, $output_id: OutputId!) {
    enableOutput(restreamId: $restream_id, id: $output_id)
  }
`;
export const DisableOutputDoc = gql`
  mutation DisableOutput($restream_id: RestreamId!, $output_id: OutputId!) {
    disableOutput(restreamId: $restream_id, id: $output_id)
  }
`;
export const EnableAllOutputsDoc = gql`
  mutation EnableAllOutputs($restream_id: RestreamId!) {
    enableAllOutputs(restreamId: $restream_id)
  }
`;
export const DisableAllOutputsDoc = gql`
  mutation DisableAllOutputs($restream_id: RestreamId!) {
    disableAllOutputs(restreamId: $restream_id)
  }
`;
export const EnableAllOutputsOfRestreamsDoc = gql`
  mutation EnableAllOutputsOfRestreams {
    enablesAllOutputsOfRestreams
  }
`;
export const DisableAllOutputsOfRestreamsDoc = gql`
  mutation DisableAllOutputsOfRestreams {
    disableAllOutputsOfRestreams
  }
`;
export const TuneVolumeDoc = gql`
  mutation TuneVolume(
    $restream_id: RestreamId!
    $output_id: OutputId!
    $mixin_id: MixinId
    $level: VolumeLevel!
    $muted: Boolean!
  ) {
    tuneVolume(
      restreamId: $restream_id
      outputId: $output_id
      mixinId: $mixin_id
      level: $level
      muted: $muted
    )
  }
`;
export const TuneDelayDoc = gql`
  mutation TuneDelay(
    $restream_id: RestreamId!
    $output_id: OutputId!
    $mixin_id: MixinId!
    $delay: Delay!
  ) {
    tuneDelay(
      restreamId: $restream_id
      outputId: $output_id
      mixinId: $mixin_id
      delay: $delay
    )
  }
`;
export const TuneSidechainDoc = gql`
  mutation TuneSidechain(
    $restream_id: RestreamId!
    $output_id: OutputId!
    $mixin_id: MixinId!
    $sidechain: Boolean!
  ) {
    tuneSidechain(
      restreamId: $restream_id
      outputId: $output_id
      mixinId: $mixin_id
      sidechain: $sidechain
    )
  }
`;
export const RemoveDvrFileDoc = gql`
  mutation RemoveDvrFile($path: String!) {
    removeDvrFile(path: $path)
  }
`;
export const SetPasswordDoc = gql`
  mutation SetPassword($new: String, $old: String, $kind: PasswordKind!) {
    setPassword(new: $new, old: $old, kind: $kind)
  }
`;
export const SetSettingsDoc = gql`
  mutation SetSettings(
    $title: String
    $deleteConfirmation: Boolean!
    $enableConfirmation: Boolean!
  ) {
    setSettings(
      title: $title
      deleteConfirmation: $deleteConfirmation
      enableConfirmation: $enableConfirmation
    )
  }
`;
export const Info = (
  options: Omit<SubscriptionOptions<InfoSubscriptionVariables>, 'query'>
) => {
  const q = client.subscribe<InfoSubscription, InfoSubscriptionVariables>({
    query: InfoDoc,
    ...options,
  });
  return q;
};
export const ServerInfo = (
  options: Omit<SubscriptionOptions<ServerInfoSubscriptionVariables>, 'query'>
) => {
  const q = client.subscribe<
    ServerInfoSubscription,
    ServerInfoSubscriptionVariables
  >({
    query: ServerInfoDoc,
    ...options,
  });
  return q;
};
export const State = (
  options: Omit<SubscriptionOptions<StateSubscriptionVariables>, 'query'>
) => {
  const q = client.subscribe<StateSubscription, StateSubscriptionVariables>({
    query: StateDoc,
    ...options,
  });
  return q;
};
export const DvrFiles = (
  options: Omit<WatchQueryOptions<DvrFilesQueryVariables>, 'query'>
): Readable<
  ApolloQueryResult<DvrFilesQuery> & {
    query: ObservableQuery<DvrFilesQuery, DvrFilesQueryVariables>;
  }
> => {
  const q = client.watchQuery({
    query: DvrFilesDoc,
    ...options,
  });
  var result = readable<
    ApolloQueryResult<DvrFilesQuery> & {
      query: ObservableQuery<DvrFilesQuery, DvrFilesQueryVariables>;
    }
  >(
    {
      data: {} as any,
      loading: true,
      error: undefined,
      networkStatus: 1,
      query: q,
    },
    (set) => {
      q.subscribe((v: any) => {
        set({ ...v, query: q });
      });
    }
  );
  return result;
};

export const ExportRestream = (
  options: Omit<WatchQueryOptions<ExportRestreamQueryVariables>, 'query'>
): Readable<
  ApolloQueryResult<ExportRestreamQuery> & {
    query: ObservableQuery<ExportRestreamQuery, ExportRestreamQueryVariables>;
  }
> => {
  const q = client.watchQuery({
    query: ExportRestreamDoc,
    ...options,
  });
  var result = readable<
    ApolloQueryResult<ExportRestreamQuery> & {
      query: ObservableQuery<ExportRestreamQuery, ExportRestreamQueryVariables>;
    }
  >(
    {
      data: {} as any,
      loading: true,
      error: undefined,
      networkStatus: 1,
      query: q,
    },
    (set) => {
      q.subscribe((v: any) => {
        set({ ...v, query: q });
      });
    }
  );
  return result;
};

export const ExportAllRestreams = (
  options: Omit<WatchQueryOptions<ExportAllRestreamsQueryVariables>, 'query'>
): Readable<
  ApolloQueryResult<ExportAllRestreamsQuery> & {
    query: ObservableQuery<
      ExportAllRestreamsQuery,
      ExportAllRestreamsQueryVariables
    >;
  }
> => {
  const q = client.watchQuery({
    query: ExportAllRestreamsDoc,
    ...options,
  });
  var result = readable<
    ApolloQueryResult<ExportAllRestreamsQuery> & {
      query: ObservableQuery<
        ExportAllRestreamsQuery,
        ExportAllRestreamsQueryVariables
      >;
    }
  >(
    {
      data: {} as any,
      loading: true,
      error: undefined,
      networkStatus: 1,
      query: q,
    },
    (set) => {
      q.subscribe((v: any) => {
        set({ ...v, query: q });
      });
    }
  );
  return result;
};

export const Import = (
  options: Omit<MutationOptions<any, ImportMutationVariables>, 'mutation'>
) => {
  const m = client.mutate<ImportMutation, ImportMutationVariables>({
    mutation: ImportDoc,
    ...options,
  });
  return m;
};
export const SetRestream = (
  options: Omit<MutationOptions<any, SetRestreamMutationVariables>, 'mutation'>
) => {
  const m = client.mutate<SetRestreamMutation, SetRestreamMutationVariables>({
    mutation: SetRestreamDoc,
    ...options,
  });
  return m;
};
export const RemoveRestream = (
  options: Omit<
    MutationOptions<any, RemoveRestreamMutationVariables>,
    'mutation'
  >
) => {
  const m = client.mutate<
    RemoveRestreamMutation,
    RemoveRestreamMutationVariables
  >({
    mutation: RemoveRestreamDoc,
    ...options,
  });
  return m;
};
export const EnableInput = (
  options: Omit<MutationOptions<any, EnableInputMutationVariables>, 'mutation'>
) => {
  const m = client.mutate<EnableInputMutation, EnableInputMutationVariables>({
    mutation: EnableInputDoc,
    ...options,
  });
  return m;
};
export const DisableInput = (
  options: Omit<MutationOptions<any, DisableInputMutationVariables>, 'mutation'>
) => {
  const m = client.mutate<DisableInputMutation, DisableInputMutationVariables>({
    mutation: DisableInputDoc,
    ...options,
  });
  return m;
};
export const SetEndpointLabel = (
  options: Omit<
    MutationOptions<any, SetEndpointLabelMutationVariables>,
    'mutation'
  >
) => {
  const m = client.mutate<
    SetEndpointLabelMutation,
    SetEndpointLabelMutationVariables
  >({
    mutation: SetEndpointLabelDoc,
    ...options,
  });
  return m;
};
export const SetOutput = (
  options: Omit<MutationOptions<any, SetOutputMutationVariables>, 'mutation'>
) => {
  const m = client.mutate<SetOutputMutation, SetOutputMutationVariables>({
    mutation: SetOutputDoc,
    ...options,
  });
  return m;
};
export const RemoveOutput = (
  options: Omit<MutationOptions<any, RemoveOutputMutationVariables>, 'mutation'>
) => {
  const m = client.mutate<RemoveOutputMutation, RemoveOutputMutationVariables>({
    mutation: RemoveOutputDoc,
    ...options,
  });
  return m;
};
export const EnableOutput = (
  options: Omit<MutationOptions<any, EnableOutputMutationVariables>, 'mutation'>
) => {
  const m = client.mutate<EnableOutputMutation, EnableOutputMutationVariables>({
    mutation: EnableOutputDoc,
    ...options,
  });
  return m;
};
export const DisableOutput = (
  options: Omit<
    MutationOptions<any, DisableOutputMutationVariables>,
    'mutation'
  >
) => {
  const m = client.mutate<
    DisableOutputMutation,
    DisableOutputMutationVariables
  >({
    mutation: DisableOutputDoc,
    ...options,
  });
  return m;
};
export const EnableAllOutputs = (
  options: Omit<
    MutationOptions<any, EnableAllOutputsMutationVariables>,
    'mutation'
  >
) => {
  const m = client.mutate<
    EnableAllOutputsMutation,
    EnableAllOutputsMutationVariables
  >({
    mutation: EnableAllOutputsDoc,
    ...options,
  });
  return m;
};
export const DisableAllOutputs = (
  options: Omit<
    MutationOptions<any, DisableAllOutputsMutationVariables>,
    'mutation'
  >
) => {
  const m = client.mutate<
    DisableAllOutputsMutation,
    DisableAllOutputsMutationVariables
  >({
    mutation: DisableAllOutputsDoc,
    ...options,
  });
  return m;
};
export const EnableAllOutputsOfRestreams = (
  options: Omit<
    MutationOptions<any, EnableAllOutputsOfRestreamsMutationVariables>,
    'mutation'
  >
) => {
  const m = client.mutate<
    EnableAllOutputsOfRestreamsMutation,
    EnableAllOutputsOfRestreamsMutationVariables
  >({
    mutation: EnableAllOutputsOfRestreamsDoc,
    ...options,
  });
  return m;
};
export const DisableAllOutputsOfRestreams = (
  options: Omit<
    MutationOptions<any, DisableAllOutputsOfRestreamsMutationVariables>,
    'mutation'
  >
) => {
  const m = client.mutate<
    DisableAllOutputsOfRestreamsMutation,
    DisableAllOutputsOfRestreamsMutationVariables
  >({
    mutation: DisableAllOutputsOfRestreamsDoc,
    ...options,
  });
  return m;
};
export const TuneVolume = (
  options: Omit<MutationOptions<any, TuneVolumeMutationVariables>, 'mutation'>
) => {
  const m = client.mutate<TuneVolumeMutation, TuneVolumeMutationVariables>({
    mutation: TuneVolumeDoc,
    ...options,
  });
  return m;
};
export const TuneDelay = (
  options: Omit<MutationOptions<any, TuneDelayMutationVariables>, 'mutation'>
) => {
  const m = client.mutate<TuneDelayMutation, TuneDelayMutationVariables>({
    mutation: TuneDelayDoc,
    ...options,
  });
  return m;
};
export const TuneSidechain = (
  options: Omit<
    MutationOptions<any, TuneSidechainMutationVariables>,
    'mutation'
  >
) => {
  const m = client.mutate<
    TuneSidechainMutation,
    TuneSidechainMutationVariables
  >({
    mutation: TuneSidechainDoc,
    ...options,
  });
  return m;
};
export const RemoveDvrFile = (
  options: Omit<
    MutationOptions<any, RemoveDvrFileMutationVariables>,
    'mutation'
  >
) => {
  const m = client.mutate<
    RemoveDvrFileMutation,
    RemoveDvrFileMutationVariables
  >({
    mutation: RemoveDvrFileDoc,
    ...options,
  });
  return m;
};
export const SetPassword = (
  options: Omit<MutationOptions<any, SetPasswordMutationVariables>, 'mutation'>
) => {
  const m = client.mutate<SetPasswordMutation, SetPasswordMutationVariables>({
    mutation: SetPasswordDoc,
    ...options,
  });
  return m;
};
export const SetSettings = (
  options: Omit<MutationOptions<any, SetSettingsMutationVariables>, 'mutation'>
) => {
  const m = client.mutate<SetSettingsMutation, SetSettingsMutationVariables>({
    mutation: SetSettingsDoc,
    ...options,
  });
  return m;
};
