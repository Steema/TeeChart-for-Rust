{*********************************************}
{  TeeChart for Rust                          }
{  Copyright (c) 2026 by Steema Software      }
{  All Rights Reserved                        }
{*********************************************}
library teechart;

{$IFDEF LINUX}
{$LIBPREFIX ''}  // teechart.so Linux
{$ENDIF}

{$IFNDEF MSWINDOWS}
{$DEFINE FMX}
{$ENDIF}

uses
  {$IFDEF FMX}
  FMXTee.Chart,
  {$ELSE}
  VCLTee.Chart,
  VCLTee.TeePNG,
  VCLTee.TeeJPEG,
  {$ENDIF }
  Math,
  System.SysUtils,
  System.Classes,
  Tee.Styles,
  Tee.RChart,
  Tee.Utils;

{$R *.res}

function TeeChartRender(nx_ptr:PInteger; x_ptr:PDouble;
                        ny_ptr:PInteger; y_ptr:PDouble;
                        nz_ptr:PInteger; z_ptr:PDouble;
                        ntexts_ptr:PInteger; texts_ptr:PPAnsiChar;
                        output_type_ptr:PPAnsiChar;
                        width_ptr, height_ptr: PInteger;
                        style_ptr, title_ptr, footer_ptr:PPAnsiChar;
                        isHoriz_ptr,
                        commander_ptr,
                        opengl_ptr,
                        bufSize :PInteger
                        ): Integer; {$IFDEF MSWINDOWS}stdcall{$ELSE}cdecl{$ENDIF}; export;
var
  outputTypeStr: string;
  tmpChart : TChart;
begin
  SetExceptionMask(exAllArithmeticExceptions);

  tmpChart:=CreateChart(RealsFrom(nx_ptr,x_ptr),
                        RealsFrom(ny_ptr,y_ptr),
                        RealsFrom(nz_ptr,z_ptr),
                        TextsFrom(ntexts_ptr,texts_ptr),
                        StyleFrom(Trim(StringFrom(style_ptr))),
                        BoolFrom(isHoriz_ptr),
                        PInteger(width_ptr)^,
                        PInteger(height_ptr)^,
                        StringFrom(title_ptr),
                        StringFrom(footer_ptr),
                        BoolFrom(opengl_ptr)
                        );
  try
    outputTypeStr := StringFrom(output_type_ptr);

    {$IFNDEF FMX} // TODO
    if SameText(outputTypeStr,'png') then
    begin
      CreateImage(tmpChart,TPNGExportFormat,bufSize^);
      result:=0;
    end
    else
    if SameText(outputTypeStr,'jpeg') or SameText(outputTypeStr,'jpg') then
    begin
      CreateImage(tmpChart,TJPEGExportFormat,bufSize^);
      result:=0;
    end
    else
    {$ENDIF}
    begin
      ShowForm(tmpChart,BoolFrom(commander_Ptr),SameText(outputTypeStr,'window'));

      tmpChart:=nil; // <-- already freed above
      result := 0;
    end;

  finally
    tmpChart.Free;
  end;
end;

exports
  TeeChartRender name 'TeeChartRender',
  TeeChartGetBytes name 'TeeChartGetBytes';

end.
