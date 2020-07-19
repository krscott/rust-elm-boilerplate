port module Main exposing (main)

import ApiTypes
import Browser
import Html
    exposing
        ( Html
        , button
        , div
        , input
        , text
        , textarea
        )
import Html.Attributes
    exposing
        ( autofocus
        , class
        , id
        , type_
        , value
        )
import Html.Events exposing (onClick, onInput)
import Json.Decode
import Json.Encode


port toRust : Json.Encode.Value -> Cmd msg


port fromRust : (Json.Encode.Value -> msg) -> Sub msg


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = subscriptions
        }



-- MODEL


type Model
    = Loading
    | EditFile String String


type Msg
    = ChangeState Model
    | SendToRust ApiTypes.ToRustMsg


init : () -> ( Model, Cmd Msg )
init _ =
    ( Loading, Cmd.none )



----- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        SendToRust toRustMsg ->
            ( model, toRust <| ApiTypes.encodeToRustMsg toRustMsg )

        ChangeState newModel ->
            ( newModel, Cmd.none )



-- VIEW


view : Model -> Html Msg
view model =
    div [ class "container" ]
        [ case model of
            Loading ->
                div [ class "centered" ] [ text "Loading..." ]

            EditFile filename content ->
                div [ id "edit-file-container" ]
                    [ div []
                        [ input
                            [ id "input-filename"
                            , type_ "text"
                            , value filename
                            , onInput (\newFilename -> ChangeState (EditFile newFilename content))
                            ]
                            []
                        ]
                    , div []
                        [ textarea
                            [ id "input-content"
                            , value content
                            , autofocus True
                            , onInput (\newContent -> ChangeState (EditFile filename newContent))
                            ]
                            []
                        ]
                    , div []
                        [ button
                            [ onClick (SendToRust (ApiTypes.UploadFile { filename = filename, content = content })) ]
                            [ text "Upload" ]
                        ]
                    ]
        ]



-- SUBSCRIPTIONS


decodeValue : Json.Encode.Value -> Msg
decodeValue json =
    let
        result =
            Json.Decode.decodeValue ApiTypes.decodeFromRustMsg json
    in
    case result of
        Ok fromRustMsg ->
            case fromRustMsg of
                ApiTypes.Loading ->
                    ChangeState Loading

                ApiTypes.EditFile { filename, content } ->
                    ChangeState (EditFile filename content)

        Err err ->
            SendToRust (ApiTypes.Log { level = ApiTypes.Error, msg = Json.Decode.errorToString err })


subscriptions : Model -> Sub Msg
subscriptions _ =
    fromRust decodeValue
